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
type BoundaryKind = "source-preflight" | "packet-run";

function requiredEnvironment(name: string): string {
	const value = process.env[name];
	if (!value) throw new Error(`missing required environment variable ${name}`);
	return value;
}

function sha256(value: string): string {
	return createHash("sha256").update(value).digest("hex");
}

function requiredSha256(name: string): string {
	const value = requiredEnvironment(name);
	if (!/^[0-9a-f]{64}$/.test(value)) throw new Error(`invalid SHA-256 in ${name}`);
	return value;
}

function parseBoundaryKind(): BoundaryKind {
	const value = process.env.NOMOS_PI_BOUNDARY_KIND ?? "source-preflight";
	if (value === "source-preflight" || value === "packet-run") return value;
	throw new Error(`invalid NOMOS_PI_BOUNDARY_KIND: ${value}`);
}

function parseWritablePaths(workspace: string, boundaryKind: BoundaryKind): string[] {
	if (boundaryKind === "source-preflight") return [];
	const value = requiredEnvironment("NOMOS_PI_WRITABLE_PATHS");
	const paths = value.split(",");
	if (paths.length !== 1 || !/^[a-z][a-z0-9_-]*$/.test(paths[0] ?? "")) {
		throw new Error("packet boundary requires exactly one safe writable directory");
	}
	for (const path of paths) {
		const resolved = realpathSync(join(workspace, path));
		if (resolved !== join(workspace, path)) {
			throw new Error(`writable path is not a direct real directory: ${path}`);
		}
	}
	return paths;
}

function sameStrings(left: readonly string[], right: readonly string[]): boolean {
	return left.length === right.length && left.every((value, index) => value === right[index]);
}

function blockBoundary(message: string): never {
	process.stderr.write(`NOMOS_PI_BOUNDARY_BLOCKED ${JSON.stringify({ error: message })}\n`);
	process.exit(78);
}

function reportedTokens(message: unknown): number | undefined {
	if (!message || typeof message !== "object") return undefined;
	const usage = (message as { usage?: unknown }).usage;
	if (!usage || typeof usage !== "object") return undefined;
	const row = usage as Record<string, unknown>;
	if (typeof row.totalTokens === "number") return row.totalTokens;
	const fields = ["input", "output", "cacheRead", "cacheWrite"];
	if (!fields.some((field) => typeof row[field] === "number")) return undefined;
	return fields.reduce((sum, field) => sum + (typeof row[field] === "number" ? (row[field] as number) : 0), 0);
}

function sandboxArguments(
	workspace: string,
	toolchainHome: string | undefined,
	boundaryKind: BoundaryKind,
	writablePaths: readonly string[],
): string[] {
	if (boundaryKind === "source-preflight" && !toolchainHome) {
		throw new Error("source preflight is missing its Rust toolchain");
	}
	const sourceDirectories =
		boundaryKind === "source-preflight" ? ["--dir", "/cargo", "--dir", "/toolchain"] : [];
	const sourceMounts =
		boundaryKind === "source-preflight"
			? ["--ro-bind", toolchainHome as string, "/toolchain", "--tmpfs", "/cargo", "--tmpfs", "/tmp"]
			: [];
	const deviceMount =
		boundaryKind === "source-preflight"
			? ["--dev", "/dev"]
			: ["--dev-bind", "/dev/null", "/dev/null"];
	const processMountPolicy =
		boundaryKind === "packet-run" ? ["--remount-ro", "/proc"] : [];
	const writableMounts =
		boundaryKind === "packet-run"
			? writablePaths.flatMap((path) => [
					"--bind",
					join(workspace, path),
					join(GUEST_WORKSPACE, path),
				])
			: [];
	const sourceEnvironment =
		boundaryKind === "source-preflight"
			? [
					"--setenv",
					"CARGO_HOME",
					"/cargo",
					"--setenv",
					"CARGO_NET_OFFLINE",
					"true",
					"--setenv",
					"CARGO_TARGET_DIR",
					"/workspace/target/pi-cold-agent",
				]
			: [];
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
		"/proc",
		"--dir",
		"/dev",
		"--dir",
		"/home",
		"--dir",
		"/home/subject",
		...sourceDirectories,
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
		boundaryKind === "source-preflight" ? "--bind" : "--ro-bind",
		workspace,
		GUEST_WORKSPACE,
		...sourceMounts,
		...writableMounts,
		"--proc",
		"/proc",
		...processMountPolicy,
		...deviceMount,
		"--setenv",
		"HOME",
		"/home/subject",
		"--setenv",
		"PATH",
		boundaryKind === "source-preflight" ? "/toolchain/bin:/usr/bin" : "/workspace/bin:/usr/bin",
		...sourceEnvironment,
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
	toolchainHome: string | undefined,
	boundaryKind: BoundaryKind,
	writablePaths: readonly string[],
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
				...sandboxArguments(workspace, toolchainHome, boundaryKind, writablePaths),
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

async function proveSandbox(
	operations: BashOperations,
	targetCommit: string,
	boundaryKind: BoundaryKind,
	writablePaths: readonly string[],
	packetManifestSha: string | undefined,
	binarySha: string | undefined,
) {
	let output = "";
	const probeSink =
		boundaryKind === "source-preflight"
			? "/tmp/.nomos-boundary-probe"
			: `/workspace/${writablePaths[0]}/.nomos-boundary-probe`;
	const common = [
		"set -eu",
		`probe_sink='${probeSink}'`,
		': > "$probe_sink"',
		'test "$PWD" = /workspace',
		"test ! -e /etc/passwd",
		"test ! -e /work/signed-dev",
		"test ! -e /run/user",
		'if touch /outside 2>"$probe_sink"; then exit 71; fi',
		"if env | cut -d= -f1 | grep -Eq '(API_KEY|AUTH_TOKEN|OAUTH_TOKEN|ACCESS_TOKEN|REFRESH_TOKEN|PASSWORD|SECRET)$'; then exit 72; fi",
		"test \"$(awk -F: 'NR > 2 {gsub(/[[:space:]]/, \"\", $1); if ($1 != \"lo\") print $1}' /proc/net/dev)\" = ''",
		'if curl --connect-timeout 1 --max-time 2 --silent http://1.1.1.1/ >"$probe_sink" 2>&1; then exit 73; fi',
	];
	const sourceChecks = [
		"test -r README.md",
		`test "$(git rev-parse HEAD)" = '${targetCommit}'`,
		"test -w /workspace",
		'cargo --version >"$probe_sink"',
	];
	const packetChecks = [
		"test -r reference/README.md",
		`test "$(cat .nomos-candidate-commit)" = '${targetCommit}'`,
		`test "$(sha256sum packet-manifest.json | cut -d' ' -f1)" = '${packetManifestSha}'`,
		`test "$(sha256sum bin/nomos | cut -d' ' -f1)" = '${binarySha}'`,
		"test -x bin/nomos",
		"test ! -e .git",
		'if touch /workspace/.nomos-undeclared-write 2>"$probe_sink"; then exit 74; fi',
		'if touch /tmp/.nomos-undeclared-write 2>"$probe_sink"; then exit 75; fi',
		'if touch /home/subject/.nomos-undeclared-write 2>"$probe_sink"; then exit 76; fi',
		"test -c /dev/null",
		"test \"$(find /dev -mindepth 1 -maxdepth 1 -printf '%p\\n')\" = /dev/null",
		"test -z \"$(cat /dev/null)\"",
		"printf nomos-boundary-probe >/dev/null",
		"test ! -e /dev/zero",
		'test ! -w /proc/self/comm',
		...writablePaths.flatMap((path) => [
			`test -w '/workspace/${path}'`,
			`touch '/workspace/${path}/.nomos-boundary-write-probe'`,
			`rm '/workspace/${path}/.nomos-boundary-write-probe'`,
		]),
		'bin/nomos --help >"$probe_sink"',
	];
	const command = [
		...common,
		...(boundaryKind === "source-preflight" ? sourceChecks : packetChecks),
		'rm "$probe_sink"',
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
	const shared = {
		targetCommitResolved: true,
		workspaceRead: true,
		outsideReadDenied: true,
		outsideWriteDenied: true,
		credentialEnvironmentAbsent: true,
		networkDenied: true,
	};
	return boundaryKind === "source-preflight"
		? { ...shared, workspaceWrite: true, cargoAvailable: true }
		: {
				...shared,
				packetManifestMatched: true,
				candidateBinaryMatched: true,
				packetRootReadOnly: true,
				temporaryStorageReadOnly: true,
				deviceFilesystemExact: true,
				deviceNullReadable: true,
				deviceNullWritable: true,
				processFilesystemReadOnly: true,
				declaredWritablePaths: [...writablePaths],
				gitMetadataAbsent: true,
			};
}

export default function nomosPiColdAgentExtension(pi: ExtensionAPI): void {
	const workspace = realpathSync(requiredEnvironment("NOMOS_PI_HOST_WORKSPACE"));
	const bwrap = realpathSync(requiredEnvironment("NOMOS_PI_BWRAP"));
	const boundaryKind = parseBoundaryKind();
	const writablePaths = parseWritablePaths(workspace, boundaryKind);
	let toolchainHome: string | undefined;
	if (boundaryKind === "source-preflight") {
		const rustupHome = realpathSync(requiredEnvironment("NOMOS_PI_RUSTUP_HOME"));
		const toolchain = requiredEnvironment("NOMOS_PI_RUST_TOOLCHAIN");
		toolchainHome = realpathSync(join(rustupHome, "toolchains", toolchain));
	}
	const expectedProvider = requiredEnvironment("NOMOS_PI_EXPECTED_PROVIDER");
	const expectedModel = requiredEnvironment("NOMOS_PI_EXPECTED_MODEL");
	const expectedThinking = requiredEnvironment("NOMOS_PI_EXPECTED_THINKING");
	const expectedPromptSha = requiredEnvironment("NOMOS_PI_SYSTEM_PROMPT_SHA256");
	const targetCommit = requiredEnvironment("NOMOS_PI_TARGET_COMMIT");
	const clientPath = requiredEnvironment("NOMOS_PI_CLIENT_PATH");
	const clientSha = requiredSha256("NOMOS_PI_CLIENT_SHA256");
	const bwrapSha = requiredSha256("NOMOS_PI_BWRAP_SHA256");
	const providerExtensionPath = requiredEnvironment("NOMOS_PI_PROVIDER_EXTENSION_PATH");
	const providerExtensionSha = requiredEnvironment("NOMOS_PI_PROVIDER_EXTENSION_SHA256");
	if (providerExtensionPath === "none" ? providerExtensionSha !== "none" : !/^[0-9a-f]{64}$/.test(providerExtensionSha)) {
		throw new Error("provider extension path/digest identity is invalid");
	}
	if (!/^[0-9a-f]{40}$/.test(targetCommit)) {
		throw new Error(`invalid target commit: ${targetCommit}`);
	}
	const packetManifestSha =
		boundaryKind === "packet-run" ? requiredSha256("NOMOS_PI_PACKET_MANIFEST_SHA256") : undefined;
	const binarySha = boundaryKind === "packet-run" ? requiredSha256("NOMOS_PI_BINARY_SHA256") : undefined;
	const taskPromptSha =
		boundaryKind === "packet-run" ? requiredSha256("NOMOS_PI_TASK_PROMPT_SHA256") : undefined;
	const taskShape = boundaryKind === "packet-run" ? requiredEnvironment("NOMOS_PI_TASK_SHAPE") : undefined;
	const operations = createIsolatedBashOperations(
		bwrap,
		workspace,
		toolchainHome,
		boundaryKind,
		writablePaths,
	);
	const bashTool = createBashTool(GUEST_WORKSPACE, {
		operations,
		exposeSessionEnvironment: true,
	});
	let boundaryReady = false;
	let assistantTurns = 0;
	let toolCalls = 0;
	let providerTokens = 0;
	let providerTokensAvailable = true;

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
			if (taskPromptSha && sha256(event.prompt) !== taskPromptSha) {
				throw new Error("task prompt digest mismatch");
			}

			const sandboxChecks = await proveSandbox(
				operations,
				targetCommit,
				boundaryKind,
				writablePaths,
				packetManifestSha,
				binarySha,
			);

			const finalSystemPrompt = event.systemPrompt.replaceAll(workspace, GUEST_WORKSPACE);
			const boundary = {
				schema: "nomos.pi_cold_agent_boundary@4",
				boundaryKind,
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
				packetManifestSha256: packetManifestSha ?? null,
				binarySha256: binarySha ?? null,
				taskPromptSha256: taskPromptSha ?? null,
				taskShape: taskShape ?? null,
				writablePaths,
				budgets: null,
				runtimeIdentity: {
					pi: { path: clientPath, sha256: clientSha },
					providerExtension:
						providerExtensionPath === "none"
							? null
							: { path: providerExtensionPath, sha256: providerExtensionSha },
					bubblewrap: { path: bwrap, sha256: bwrapSha },
				},
				sandbox: {
					backend: "bubblewrap",
					binary: bwrap,
					root: "read-only",
					workspace:
						boundaryKind === "source-preflight"
							? "read-write-only-host-mount"
							: "read-only-packet-with-declared-writable-paths",
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

	pi.on("tool_call", () => {
		if (boundaryKind !== "packet-run") return;
		toolCalls += 1;
	});

	pi.on("turn_start", () => {
		if (boundaryKind !== "packet-run") return;
		assistantTurns += 1;
	});

	pi.on("message_end", (event) => {
		if (boundaryKind !== "packet-run" || event.message.role !== "assistant") return;
		const tokens = reportedTokens(event.message);
		if (tokens === undefined) {
			providerTokensAvailable = false;
			return;
		}
		providerTokens += tokens;
	});

	pi.on("agent_end", () => {
		if (boundaryKind !== "packet-run") return;
		process.stderr.write(
			`NOMOS_PI_ACCOUNTING ${JSON.stringify({
				assistantTurns,
				toolCalls,
				providerReportedTokens: providerTokensAvailable ? providerTokens : null,
			})}\n`,
		);
	});

	pi.on("before_provider_request", () => {
		if (boundaryReady) return;
		blockBoundary("provider request preceded boundary proof");
	});
}
