use std::collections::HashMap;

use super::*;
use crate::{analysis::store::declaration::DeclarationType, lexer::token::Span};

impl Analyzer<'_> {
    pub fn register_packing(&mut self, packing: &Packing) {
        let packing_iid = self.idents.insert(packing.ident.0);
        let packing_key = Key::new(self.scope, packing_iid);
        let packing_did = self.declarations.get(packing_key).unwrap();
        let packing_scope = self.declarations.scope_from_id(packing_did);
        self.enter_scope(packing_scope);

        let mut fields = HashMap::new();

        for field in &packing.fields {
            let (iid, sid) = self.register_field(field);
            fields.insert(iid, sid);
        }

        self.declarations.initialize(packing_did, |info| {
            if let DeclarationType::Packing(ref mut packing) = info.ty {
                packing.set_fields(fields);
            }
        });

        self.exit_scope();
    }

    pub fn register_aor(&mut self, aor: &Aor) {
        let aor_iid = self.idents.insert(aor.ident.0);
        let aor_key = Key::new(self.scope, aor_iid);
        let aor_did = self.declarations.get(aor_key).unwrap();
        let aor_scope = self.declarations.scope_from_id(aor_did);

        self.enter_scope(aor_scope);

        let mut variants = HashMap::new();
        for variant in &aor.variants {
            let (span, symbol_type) = match variant {
                Variant::Field(field) => (field.ident.0, self.symbol_type(&field.ty)),
                Variant::SpannedIdent(ident) => (ident.0, SymbolType::Variant),
            };
            let (iid, sid) = self.register_variant(span, symbol_type);
            variants.insert(iid, sid);
        }

        self.declarations.initialize(aor_did, |info| {
            if let DeclarationType::Aor(ref mut aor) = info.ty {
                aor.set_variants(variants);
            }
        });

        self.exit_scope();
    }

    ///doesnt check if the field is already initialized, so the caller must make sure thats
    ///not the case
    ///NOTE: calls get_sinfo once, thus moving the 'at' pointer for field declaration by 1
    pub fn register_field(&mut self, field: &Field) -> (IdentId, SymbolId) {
        let field_span = field.ident.0;
        let field_iid = self.idents.insert(field_span);
        let field_key = SymbolKey::new(self.scope, field_iid);

        if let Some(field_sid) = self.symbols.get(field_key) {
            let declared_span = self.symbols.refer(field_sid).span.into();
            self.errors.push(
                AnalysisError::DuplicateField {
                    declared_span,
                    duplicate_span: field_span.into(),
                }
                .into(),
            );
        }

        let field_type = self.symbol_type(&field.ty);
        let field_sid = self.symbols.insert(field_key, field_span, field_type);

        (field_iid, field_sid)
    }

    pub fn register_variant(
        &mut self,
        variant_span: Span,
        symbol_type: SymbolType,
    ) -> (IdentId, SymbolId) {
        let variant_iid = self.idents.insert(variant_span);
        let variant_key = SymbolKey::new(self.scope, variant_iid);

        if let Some(variant_sid) = self.symbols.get(variant_key) {
            let declared_span = self.symbols.refer(variant_sid).span.into();
            self.errors.push(
                AnalysisError::DuplicateVariant {
                    declared_span,
                    duplicate_span: variant_span.into(),
                }
                .into(),
            );
        }

        let variant_sid = self.symbols.insert(variant_key, variant_span, symbol_type);
        (variant_iid, variant_sid)
    }
}
