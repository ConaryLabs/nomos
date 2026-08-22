import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { realpathSync } from "node:fs";
import { join } from "node:path";

import {
	createBashTool,
	type BashOperations,
	type ExtensionAPI,
} from "@earendil-works/pi-coding-agent";

const GUEST_WORKSPACE = "/workspace";
const EXPECTED_ACTIVE_TOOLS = ["bash"];
const EXPOSED_SESSION_ENVIRONMENT = new Set([
	"PI_MODEL",
	"PI_PROVIDER",
	"PI_REASONING_LEVEL",
	"PI_SESSION_ID",
]);

function requiredEnvironment(name: string): string {
	const value = process.env[name];
	if (!value) throw new Error(`missing required environment variable ${name}`);
	return value;
}

function sha256(value: string): string {
	return createHash("sha256").update(value).digest("hex");
}

function sameStrings(left: readonly string[], right: readonly string[]): boolean {
	return left.length === right.length && left.every((value, index) => value === right[index]);
}

function blockBoundary(message: string): never {
	process.stderr.write(`NOMOS_PI_BOUNDARY_BLOCKED ${JSON.stringify({ error: message })}\n`);
	process.exit(78);
}

function sandboxArguments(workspace: string, toolchainHome: string): string[] {
	return [
		"--die-with-parent",
		"--new-session",
		"--unshare-all",
		"--clearenv",
		"--dir",
		"/usr",
		"--dir",
		GUEST_WORKSPACE,
		"--dir",
		"/tmp",
		"--dir",
		"/cargo",
		"--dir",
		"/toolchain",
		"--dir",
		"/proc",
		"--dir",
		"/dev",
		"--dir",
		"/home",
		"--dir",
		"/home/subject",
		"--symlink",
		"usr/bin",
		"/bin",
		"--symlink",
		"usr/lib",
		"/lib",
		"--symlink",
		"usr/lib",
		"/lib64",
		"--remount-ro",
		"/",
		"--ro-bind",
		"/usr",
		"/usr",
		"--bind",
		workspace,
		GUEST_WORKSPACE,
		"--ro-bind",
		toolchainHome,
		"/toolchain",
		"--tmpfs",
		"/tmp",
		"--tmpfs",
		"/cargo",
		"--proc",
		"/proc",
		"--dev",
		"/dev",
		"--setenv",
		"HOME",
		"/home/subject",
		"--setenv",
		"PATH",
		"/toolchain/bin:/usr/bin",
		"--setenv",
		"CARGO_HOME",
		"/cargo",
		"--setenv",
		"CARGO_NET_OFFLINE",
		"true",
		"--setenv",
		"CARGO_TARGET_DIR",
		"/workspace/target/pi-cold-agent",
		"--setenv",
		"LC_ALL",
		"C.UTF-8",
		"--setenv",
		"TERM",
		"dumb",
		"--chdir",
		GUEST_WORKSPACE,
	];
}

function createIsolatedBashOperations(
	bwrap: string,
	workspace: string,
	toolchainHome: string,
): BashOperations {
	return {
		exec: async (command, cwd, options) => {
			if (cwd !== GUEST_WORKSPACE) {
				throw new Error(`refusing unexpected guest working directory: ${cwd}`);
			}

			const exposedSessionEnvironment = Object.entries(options.env ?? {})
				.filter(([name, value]) => EXPOSED_SESSION_ENVIRONMENT.has(name) && typeof value === "string")
				.map(([name, value]) => ["--setenv", name, value as string])
				.flat();
			const args = [
				...sandboxArguments(workspace, toolchainHome),
				...exposedSessionEnvironment,
				"/bin/bash",
				"-lc",
				command,
			];

			return await new Promise<{ exitCode: number | null }>((resolve, reject) => {
				const child = spawn(bwrap, args, {
					cwd: "/",
					env: { PATH: "/usr/bin:/bin" },
					stdio: ["ignore", "pipe", "pipe"],
				});
				let timedOut = false;
				const timeout =
					options.timeout && options.timeout > 0
						? setTimeout(() => {
								timedOut = true;
								child.kill("SIGTERM");
							}, options.timeout * 1000)
						: undefined;
				const abort = () => child.kill("SIGTERM");
				options.signal?.addEventListener("abort", abort, { once: true });
				child.stdout.on("data", options.onData);
				child.stderr.on("data", options.onData);
				child.once("error", reject);
				child.once("close", (code) => {
					if (timeout) clearTimeout(timeout);
					options.signal?.removeEventListener("abort", abort);
					if (timedOut) {
						reject(new Error(`sandbox command timed out after ${options.timeout}s`));
						return;
					}
					resolve({ exitCode: code });
				});
			});
		},
	};
}

async function proveSandbox(operations: BashOperations, targetCommit: string) {
	let output = "";
	const command = [
		"set -eu",
		'test "$PWD" = /workspace',
		`test "$(git rev-parse HEAD)" = '${targetCommit}'`,
		"test -r README.md",
		"test -w /workspace",
		"test ! -e /etc/passwd",
		"test ! -e /work/signed-dev",
		"test ! -e /run/user",
		"if touch /outside 2>/dev/null; then exit 71; fi",
		"if env | cut -d= -f1 | grep -Eq '(API_KEY|AUTH_TOKEN|OAUTH_TOKEN|ACCESS_TOKEN|REFRESH_TOKEN|PASSWORD|SECRET)$'; then exit 72; fi",
		"test \"$(awk -F: 'NR > 2 {gsub(/[[:space:]]/, \"\", $1); if ($1 != \"lo\") print $1}' /proc/net/dev)\" = ''",
		"if curl --connect-timeout 1 --max-time 2 --silent http://1.1.1.1/ >/dev/null 2>&1; then exit 73; fi",
		"cargo --version >/dev/null",
		"printf 'NOMOS_PI_SANDBOX_SELF_TEST PASS\\n'",
	].join("\n");
	const result = await operations.exec(command, GUEST_WORKSPACE, {
		onData: (chunk) => {
			output += chunk.toString("utf8");
		},
		timeout: 15,
		env: process.env,
	});
	if (result.exitCode !== 0 || output !== "NOMOS_PI_SANDBOX_SELF_TEST PASS\n") {
		throw new Error(`sandbox self-test failed with exit ${result.exitCode}: ${output.trim()}`);
	}
	return {
		targetCommitResolved: true,
		workspaceRead: true,
		workspaceWrite: true,
		outsideReadDenied: true,
		outsideWriteDenied: true,
		credentialEnvironmentAbsent: true,
		networkDenied: true,
		cargoAvailable: true,
	};
}

export default function nomosPiColdAgentExtension(pi: ExtensionAPI): void {
	const workspace = realpathSync(requiredEnvironment("NOMOS_PI_HOST_WORKSPACE"));
	const rustupHome = realpathSync(requiredEnvironment("NOMOS_PI_RUSTUP_HOME"));
	const bwrap = realpathSync(requiredEnvironment("NOMOS_PI_BWRAP"));
	const toolchain = requiredEnvironment("NOMOS_PI_RUST_TOOLCHAIN");
	const toolchainHome = realpathSync(join(rustupHome, "toolchains", toolchain));
	const expectedProvider = requiredEnvironment("NOMOS_PI_EXPECTED_PROVIDER");
	const expectedModel = requiredEnvironment("NOMOS_PI_EXPECTED_MODEL");
	const expectedThinking = requiredEnvironment("NOMOS_PI_EXPECTED_THINKING");
	const expectedPromptSha = requiredEnvironment("NOMOS_PI_SYSTEM_PROMPT_SHA256");
	const targetCommit = requiredEnvironment("NOMOS_PI_TARGET_COMMIT");
	if (!/^[0-9a-f]{40}$/.test(targetCommit)) {
		throw new Error(`invalid target commit: ${targetCommit}`);
	}
	const operations = createIsolatedBashOperations(bwrap, workspace, toolchainHome);
	const bashTool = createBashTool(GUEST_WORKSPACE, {
		operations,
		exposeSessionEnvironment: true,
	});
	let boundaryReady = false;

	pi.registerTool({
		...bashTool,
		execute: async (id, params, signal, onUpdate) =>
			await bashTool.execute(id, params, signal, onUpdate),
	});

	pi.on("before_agent_start", async (event, ctx) => {
		try {
			const activeTools = [...pi.getActiveTools()].sort();
			const configuredTools = pi
				.getAllTools()
				.map((tool) => ({ name: tool.name, source: tool.sourceInfo }))
				.sort((left, right) => left.name.localeCompare(right.name));
			const contextFiles = event.systemPromptOptions.contextFiles ?? [];
			const skills = event.systemPromptOptions.skills ?? [];
			const sessionFile = ctx.sessionManager.getSessionFile();
			const entries = ctx.sessionManager.getEntries();
			const model = ctx.model;
			const thinking = ctx.thinkingLevel ?? pi.getThinkingLevel();

			if (ctx.mode !== "json") throw new Error(`expected JSON mode, got ${ctx.mode}`);
			if (ctx.isProjectTrusted()) throw new Error("project-local resources are trusted");
			if (sessionFile !== undefined) {
				throw new Error(`session unexpectedly persisted at ${sessionFile}`);
			}
			if (
				entries.length !== 2 ||
				entries[0]?.type !== "model_change" ||
				entries[0].provider !== expectedProvider ||
				entries[0].modelId !== expectedModel ||
				entries[1]?.type !== "thinking_level_change" ||
				entries[1].thinkingLevel !== expectedThinking
			) {
				throw new Error(
					`fresh session metadata is unexpected: ${entries.map((entry) => entry.type).join(",")}`,
				);
			}
			const header = ctx.sessionManager.getHeader();
			if (!header || header.parentSession !== undefined || header.cwd !== workspace) {
				throw new Error("fresh session header is missing, parented, or in the wrong worktree");
			}
			if (!sameStrings(activeTools, EXPECTED_ACTIVE_TOOLS)) {
				throw new Error(`effective tools are not exactly bash: ${activeTools.join(",")}`);
			}
			if (contextFiles.length !== 0) throw new Error(`loaded ${contextFiles.length} context files`);
			if (skills.length !== 0) throw new Error(`loaded ${skills.length} skills`);
			if (!model || model.provider !== expectedProvider || model.id !== expectedModel) {
				throw new Error(`resolved model mismatch: ${model?.provider ?? "none"}/${model?.id ?? "none"}`);
			}
			if (thinking !== expectedThinking) {
				throw new Error(`resolved thinking mismatch: ${thinking}`);
			}
			if (sha256(event.systemPromptOptions.customPrompt ?? "") !== expectedPromptSha) {
				throw new Error("custom system prompt digest mismatch");
			}

			const sandboxChecks = await proveSandbox(operations, targetCommit);

			const finalSystemPrompt = event.systemPrompt.replaceAll(workspace, GUEST_WORKSPACE);
			const boundary = {
				schema: "nomos.pi_cold_agent_boundary@1",
				mode: ctx.mode,
				targetCommit,
				hostWorkspace: workspace,
				guestWorkspace: GUEST_WORKSPACE,
				provider: model.provider,
				model: model.id,
				thinking,
				sessionId: ctx.sessionManager.getSessionId(),
				sessionFile: null,
				projectTrusted: false,
				entryTypesBeforeRun: entries.map((entry) => entry.type),
				activeTools,
				configuredTools,
				contextFiles: [],
				skills: [],
				systemPromptSha256: expectedPromptSha,
				finalSystemPromptSha256: sha256(finalSystemPrompt),
				sandbox: {
					backend: "bubblewrap",
					binary: bwrap,
					root: "read-only",
					workspace: "read-write-only-host-mount",
					network: "unshared",
					environment: "cleared-and-allowlisted",
					checks: sandboxChecks,
					selfTest: "pass",
				},
			};
			process.stderr.write(`NOMOS_PI_BOUNDARY ${JSON.stringify(boundary)}\n`);
			boundaryReady = true;

			return {
				systemPrompt: finalSystemPrompt,
			};
		} catch (error) {
			const message = error instanceof Error ? error.message : String(error);
			blockBoundary(message);
		}
	});

	pi.on("before_provider_request", () => {
		if (boundaryReady) return;
		blockBoundary("provider request preceded boundary proof");
	});
}
