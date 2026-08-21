//! Hand-written parser for `nomos.source@1`.

use nomos_core::{
    CatalogValueId, Diagnostic, EntityId, Ident, PrimitiveKindId, RepairClass, SchemaId,
    SourcePath, SourceSpan,
};
use nomos_schema::{
    Binding, Cell, Direction, ForbiddenFactOwner, SourceDocument, SourceEntity, SourceField,
    SourceRelation, Spanned,
};

use crate::diagnostics;

#[derive(Clone, Copy, Debug)]
struct Token<'a> {
    text: &'a str,
    byte_start: usize,
    byte_end: usize,
    column: usize,
}

#[derive(Clone, Copy, Debug)]
struct Line<'a> {
    raw: &'a str,
    byte_start: usize,
    byte_end: usize,
    number: usize,
}

impl<'a> Line<'a> {
    fn tokens(self) -> Vec<Token<'a>> {
        let bytes = self.raw.as_bytes();
        let mut tokens = Vec::new();
        let mut cursor = 0;
        while cursor < bytes.len() {
            while cursor < bytes.len() && matches!(bytes[cursor], b' ' | b'\t') {
                cursor += 1;
            }
            if cursor == bytes.len() {
                break;
            }
            let start = cursor;
            while cursor < bytes.len() && !matches!(bytes[cursor], b' ' | b'\t') {
                cursor += 1;
            }
            tokens.push(Token {
                text: &self.raw[start..cursor],
                byte_start: self.byte_start + start,
                byte_end: self.byte_start + cursor,
                column: start + 1,
            });
        }
        tokens
    }

    fn is_ignored(self) -> bool {
        let trimmed = self.raw.trim_start_matches([' ', '\t']);
        trimmed.is_empty() || trimmed.starts_with('#')
    }
}

pub(crate) fn parse(source: &str, path: SourcePath) -> Result<SourceDocument, Diagnostic> {
    if source.len() > u32::MAX as usize {
        return Err(Diagnostic::new(
            diagnostics::SOURCE_TOO_LARGE,
            "source exceeds the 32-bit byte range used by source spans",
        ));
    }

    let lines = split_lines(source);
    let mut schema = None;
    let mut catalog_values = Vec::new();
    let mut entities = Vec::new();
    let mut relations = Vec::new();
    let mut forbidden_fact_owners = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        let line = lines[index];
        if line.is_ignored() {
            index += 1;
            continue;
        }
        let tokens = line.tokens();
        if schema.is_none() && tokens[0].text != "schema" {
            return Err(on_line(
                Diagnostic::new(
                    diagnostics::SOURCE_SCHEMA_REQUIRED,
                    "the first declaration must be `schema nomos.source@1`",
                )
                .with_repair(RepairClass::FixSourceSyntax),
                &path,
                line,
            ));
        }

        match tokens[0].text {
            "schema" => {
                exact_arity(&tokens, 2, &path, line, "schema <schema-id>")?;
                if schema.is_some() {
                    return Err(on_line(
                        Diagnostic::new(
                            diagnostics::SOURCE_SCHEMA_REQUIRED,
                            "the source schema may be declared exactly once",
                        )
                        .with_repair(RepairClass::RemoveDuplicateDeclaration),
                        &path,
                        line,
                    ));
                }
                let parsed =
                    with_token_span(SchemaId::parse(tokens[1].text), &path, line, tokens[1])?;
                if parsed != nomos_schema::source_schema() {
                    return Err(Diagnostic::new(
                        diagnostics::SOURCE_SCHEMA_UNSUPPORTED,
                        format!(
                            "source schema `{parsed}` is unsupported; expected `{}`",
                            nomos_schema::source_schema()
                        ),
                    )
                    .with_span(token_span(&path, line, tokens[1]))
                    .with_repair(RepairClass::FixSourceSyntax));
                }
                schema = Some(Spanned::new(parsed, token_span(&path, line, tokens[1])));
                index += 1;
            }
            "catalog" => {
                exact_arity(&tokens, 2, &path, line, "catalog <catalog>/<value>")?;
                let value = with_token_span(
                    CatalogValueId::parse(tokens[1].text),
                    &path,
                    line,
                    tokens[1],
                )?;
                catalog_values.push(Spanned::new(value, token_span(&path, line, tokens[1])));
                index += 1;
            }
            "entity" => {
                let (entity, next) = parse_entity(&lines, index, &path)?;
                entities.push(entity);
                index = next;
            }
            "relation" => {
                exact_arity(
                    &tokens,
                    4,
                    &path,
                    line,
                    "relation <subject> <kind> <object>",
                )?;
                let subject =
                    with_token_span(EntityId::parse(tokens[1].text), &path, line, tokens[1])?;
                let kind = with_token_span(Ident::new(tokens[2].text), &path, line, tokens[2])?;
                let object =
                    with_token_span(EntityId::parse(tokens[3].text), &path, line, tokens[3])?;
                relations.push(SourceRelation::new(
                    Spanned::new(subject, token_span(&path, line, tokens[1])),
                    Spanned::new(kind, token_span(&path, line, tokens[2])),
                    Spanned::new(object, token_span(&path, line, tokens[3])),
                    line_span(&path, line),
                ));
                index += 1;
            }
            "fact_owner" => {
                exact_arity(&tokens, 3, &path, line, "fact_owner <fact-class> <owner>")?;
                forbidden_fact_owners.push(ForbiddenFactOwner::new(
                    tokens[1].text.to_owned(),
                    tokens[2].text.to_owned(),
                    line_span(&path, line),
                ));
                index += 1;
            }
            "end" => {
                return Err(on_line(
                    Diagnostic::new(
                        diagnostics::SOURCE_SYNTAX,
                        "`end` has no open entity declaration",
                    )
                    .with_repair(RepairClass::FixSourceSyntax),
                    &path,
                    line,
                ));
            }
            unknown => {
                return Err(on_line(
                    Diagnostic::new(
                        diagnostics::SOURCE_UNKNOWN_STATEMENT,
                        format!("unknown top-level statement `{unknown}`"),
                    )
                    .with_repair(RepairClass::FixSourceSyntax),
                    &path,
                    line,
                ));
            }
        }
    }

    let schema = schema.ok_or_else(|| {
        Diagnostic::new(
            diagnostics::SOURCE_SCHEMA_REQUIRED,
            "source is empty; expected `schema nomos.source@1`",
        )
        .with_span(SourceSpan::new(path.clone(), 0, 0, 1, 1).expect("empty span is valid"))
        .with_repair(RepairClass::FixSourceSyntax)
    })?;

    Ok(SourceDocument::new(
        schema,
        catalog_values,
        entities,
        relations,
        forbidden_fact_owners,
    ))
}

fn parse_entity(
    lines: &[Line<'_>],
    header_index: usize,
    path: &SourcePath,
) -> Result<(SourceEntity, usize), Diagnostic> {
    let header = lines[header_index];
    let tokens = header.tokens();
    exact_arity(
        &tokens,
        3,
        path,
        header,
        "entity <entity-id> primitive/<kind>",
    )?;
    let id = with_token_span(EntityId::parse(tokens[1].text), path, header, tokens[1])?;
    let primitive = with_token_span(
        PrimitiveKindId::parse(tokens[2].text),
        path,
        header,
        tokens[2],
    )?;
    let id = Spanned::new(id, token_span(path, header, tokens[1]));
    let primitive = Spanned::new(primitive, token_span(path, header, tokens[2]));

    let mut fields = Vec::new();
    let mut index = header_index + 1;
    while index < lines.len() {
        let line = lines[index];
        if line.is_ignored() {
            index += 1;
            continue;
        }
        let tokens = line.tokens();
        if tokens[0].text == "end" {
            exact_arity(&tokens, 1, path, line, "end")?;
            let span = SourceSpan::new(
                path.clone(),
                u32::try_from(header.byte_start).expect("source length was checked"),
                u32::try_from(line.byte_end).expect("source length was checked"),
                u32::try_from(header.number).expect("line count fits source length"),
                1,
            )
            .expect("parser creates a forward source span");
            return Ok((SourceEntity::new(id, primitive, fields, span), index + 1));
        }
        fields.push(parse_field(&tokens, path, line)?);
        index += 1;
    }

    Err(on_line(
        Diagnostic::new(
            diagnostics::SOURCE_UNCLOSED_ENTITY,
            format!("entity `{}` reaches end of file without `end`", id.value()),
        )
        .with_repair(RepairClass::FixSourceSyntax),
        path,
        header,
    ))
}

fn parse_field(
    tokens: &[Token<'_>],
    path: &SourcePath,
    line: Line<'_>,
) -> Result<SourceField, Diagnostic> {
    match tokens[0].text {
        "anchor" => parse_anchor(tokens, path, line),
        "credential" => {
            exact_arity(tokens, 2, path, line, "credential <catalog>/<value>")?;
            let value =
                with_token_span(CatalogValueId::parse(tokens[1].text), path, line, tokens[1])?;
            Ok(SourceField::Credential(Spanned::new(
                value,
                line_span(path, line),
            )))
        }
        "lattice_relation" => {
            exact_arity(tokens, 3, path, line, "lattice_relation <kind> <entity>")?;
            let relation = with_token_span(Ident::new(tokens[1].text), path, line, tokens[1])?;
            let target = with_token_span(EntityId::parse(tokens[2].text), path, line, tokens[2])?;
            Ok(SourceField::LatticeRelation {
                relation: Spanned::new(relation, token_span(path, line, tokens[1])),
                target: Spanned::new(target, token_span(path, line, tokens[2])),
                span: line_span(path, line),
            })
        }
        "transform" => Ok(SourceField::RawTransform(line_span(path, line))),
        "derived" => {
            if tokens.len() < 2 {
                return Err(arity_error(path, line, "derived <fact-name> <value...>"));
            }
            Ok(SourceField::DerivedFact {
                name: tokens[1].text.to_owned(),
                span: line_span(path, line),
            })
        }
        unknown => Err(on_line(
            Diagnostic::new(
                diagnostics::SOURCE_UNKNOWN_STATEMENT,
                format!("unknown entity field `{unknown}`"),
            )
            .with_repair(RepairClass::FixSourceSyntax),
            path,
            line,
        )),
    }
}

fn parse_anchor(
    tokens: &[Token<'_>],
    path: &SourcePath,
    line: Line<'_>,
) -> Result<SourceField, Diagnostic> {
    let binding = match tokens.get(1).map(|token| token.text) {
        Some("cell") => {
            exact_arity(tokens, 5, path, line, "anchor cell <x> <y> <z>")?;
            Binding::Cell(parse_cell(&tokens[2..5], path, line)?)
        }
        Some("face") => {
            exact_arity(tokens, 6, path, line, "anchor face <x> <y> <z> <direction>")?;
            let cell = parse_cell(&tokens[2..5], path, line)?;
            let direction = Direction::parse(tokens[5].text).ok_or_else(|| {
                on_line(
                    Diagnostic::new(
                        diagnostics::SOURCE_SYNTAX,
                        format!("`{}` is not a lattice face direction", tokens[5].text),
                    )
                    .with_repair(RepairClass::FixSourceSyntax),
                    path,
                    line,
                )
            })?;
            Binding::Face { cell, direction }
        }
        Some("region") => {
            exact_arity(
                tokens,
                8,
                path,
                line,
                "anchor region <min-x> <min-y> <min-z> <max-x> <max-y> <max-z>",
            )?;
            Binding::Region {
                min: parse_cell(&tokens[2..5], path, line)?,
                max: parse_cell(&tokens[5..8], path, line)?,
            }
        }
        _ => return Err(arity_error(path, line, "anchor <cell|face|region> ...")),
    };
    Ok(SourceField::Anchor(Spanned::new(
        binding,
        line_span(path, line),
    )))
}

fn parse_cell(tokens: &[Token<'_>], path: &SourcePath, line: Line<'_>) -> Result<Cell, Diagnostic> {
    let mut values = [0_i32; 3];
    for (index, token) in tokens.iter().enumerate() {
        if token.text.starts_with('+')
            || token.text.is_empty()
            || token.text == "-"
            || !token
                .text
                .trim_start_matches('-')
                .bytes()
                .all(|byte| byte.is_ascii_digit())
        {
            return Err(integer_error(path, line, *token));
        }
        values[index] = token
            .text
            .parse::<i32>()
            .map_err(|_| integer_error(path, line, *token))?;
    }
    Ok(Cell::new(values[0], values[1], values[2]))
}

fn integer_error(path: &SourcePath, line: Line<'_>, token: Token<'_>) -> Diagnostic {
    with_token_span::<()>(
        Err(Diagnostic::new(
            diagnostics::SOURCE_INTEGER_INVALID,
            format!("`{}` is not a signed 32-bit decimal integer", token.text),
        )
        .with_repair(RepairClass::FixSourceSyntax)),
        path,
        line,
        token,
    )
    .expect_err("this helper always constructs an error")
}

fn exact_arity(
    tokens: &[Token<'_>],
    expected: usize,
    path: &SourcePath,
    line: Line<'_>,
    shape: &str,
) -> Result<(), Diagnostic> {
    if tokens.len() == expected {
        Ok(())
    } else {
        Err(arity_error(path, line, shape))
    }
}

fn arity_error(path: &SourcePath, line: Line<'_>, shape: &str) -> Diagnostic {
    on_line(
        Diagnostic::new(
            diagnostics::SOURCE_SYNTAX,
            format!("statement does not match `{shape}`"),
        )
        .with_repair(RepairClass::FixSourceSyntax),
        path,
        line,
    )
}

fn with_token_span<T>(
    result: Result<T, Diagnostic>,
    path: &SourcePath,
    line: Line<'_>,
    token: Token<'_>,
) -> Result<T, Diagnostic> {
    result.map_err(|diagnostic| diagnostic.with_span(token_span(path, line, token)))
}

fn on_line(diagnostic: Diagnostic, path: &SourcePath, line: Line<'_>) -> Diagnostic {
    diagnostic.with_span(line_span(path, line))
}

fn token_span(path: &SourcePath, line: Line<'_>, token: Token<'_>) -> SourceSpan {
    SourceSpan::new(
        path.clone(),
        u32::try_from(token.byte_start).expect("source length was checked"),
        u32::try_from(token.byte_end).expect("source length was checked"),
        u32::try_from(line.number).expect("line count fits source length"),
        u32::try_from(token.column).expect("column fits source length"),
    )
    .expect("parser creates a forward source span")
}

fn line_span(path: &SourcePath, line: Line<'_>) -> SourceSpan {
    let leading = line.raw.len() - line.raw.trim_start_matches([' ', '\t']).len();
    SourceSpan::new(
        path.clone(),
        u32::try_from(line.byte_start + leading).expect("source length was checked"),
        u32::try_from(line.byte_end).expect("source length was checked"),
        u32::try_from(line.number).expect("line count fits source length"),
        u32::try_from(leading + 1).expect("column fits source length"),
    )
    .expect("parser creates a forward source span")
}

fn split_lines(source: &str) -> Vec<Line<'_>> {
    let mut lines = Vec::new();
    let mut byte_start = 0;
    for (index, terminated) in source.split_inclusive('\n').enumerate() {
        let raw = terminated
            .strip_suffix('\n')
            .unwrap_or(terminated)
            .strip_suffix('\r')
            .unwrap_or_else(|| terminated.strip_suffix('\n').unwrap_or(terminated));
        lines.push(Line {
            raw,
            byte_start,
            byte_end: byte_start + raw.len(),
            number: index + 1,
        });
        byte_start += terminated.len();
    }
    lines
}
