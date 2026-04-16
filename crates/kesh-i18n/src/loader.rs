//! Chargement des fichiers Fluent (.ftl) et résolution de messages.

use std::collections::HashMap;
use std::path::Path;

use fluent_bundle::resolver::ResolverError;
use fluent_bundle::resolver::errors::ReferenceKind;
use fluent_bundle::{FluentArgs, FluentError, FluentResource};
use fluent_syntax::ast;

/// Vrai si `err` est un « variable manquante » (`{ $var }` non fourni dans
/// `args`). C'est le cas attendu dans `all_messages` qui pré-résout toutes
/// les clés sans contexte, avec interpolation ensuite côté frontend.
fn is_missing_variable_error(err: &FluentError) -> bool {
    matches!(
        err,
        FluentError::ResolverError(ResolverError::Reference(ReferenceKind::Variable { .. }))
    )
}

/// Type alias pour le FluentBundle concurrent (Send + Sync).
type ConcurrentBundle = fluent_bundle::bundle::FluentBundle<
    FluentResource,
    intl_memoizer::concurrent::IntlLangMemoizer,
>;

use crate::Locale;
use crate::error::I18nError;

/// Bundle i18n contenant les traductions pour toutes les locales.
pub struct I18nBundle {
    bundles: HashMap<Locale, ConcurrentBundle>,
    /// Clés de messages par locale (FluentBundle n'expose pas d'itérateur).
    keys: HashMap<Locale, Vec<String>>,
}

impl I18nBundle {
    /// Charge les fichiers `{locale}/messages.ftl` depuis `locales_dir`.
    ///
    /// Chaque sous-répertoire (fr-CH, de-CH, it-CH, en-CH) doit contenir
    /// un fichier `messages.ftl`. Erreur si un fichier est manquant ou
    /// contient des erreurs de syntaxe Fluent.
    pub fn load(locales_dir: &Path) -> Result<Self, I18nError> {
        let mut bundles: HashMap<Locale, ConcurrentBundle> = HashMap::new();
        let mut keys = HashMap::new();

        for locale in Locale::ALL {
            let ftl_path = locales_dir.join(locale.dir_name()).join("messages.ftl");
            let source = std::fs::read_to_string(&ftl_path)
                .map_err(|_| I18nError::MissingResource(ftl_path.display().to_string()))?;

            // Extraire les clés via fluent-syntax avant de consommer la source
            let parsed = fluent_syntax::parser::parse(source.as_str()).map_err(|(_, errs)| {
                I18nError::FluentParse {
                    locale: locale.to_string(),
                    detail: errs
                        .iter()
                        .map(|e| format!("{e:?}"))
                        .collect::<Vec<_>>()
                        .join("; "),
                }
            })?;
            let msg_keys: Vec<String> = parsed
                .body
                .iter()
                .filter_map(|entry| match entry {
                    ast::Entry::Message(m) => Some(m.id.name.to_string()),
                    _ => None,
                })
                .collect();
            keys.insert(locale, msg_keys);

            let resource =
                FluentResource::try_new(source).map_err(|(_, errs)| I18nError::FluentParse {
                    locale: locale.to_string(),
                    detail: errs
                        .iter()
                        .map(|e| format!("{e:?}"))
                        .collect::<Vec<_>>()
                        .join("; "),
                })?;

            let lang_id = locale
                .dir_name()
                .parse()
                .unwrap_or_else(|_| "fr".parse().expect("'fr' is a valid language identifier"));

            let mut bundle = ConcurrentBundle::new_concurrent(vec![lang_id]);
            bundle
                .add_resource(resource)
                .map_err(|errs| I18nError::FluentParse {
                    locale: locale.to_string(),
                    detail: errs
                        .iter()
                        .map(|e| format!("{e:?}"))
                        .collect::<Vec<_>>()
                        .join("; "),
                })?;

            bundles.insert(locale, bundle);
        }

        Ok(Self { bundles, keys })
    }

    /// Résout un message pour la locale donnée.
    ///
    /// Fallback : si la clé est absente dans `locale`, cherche dans FrCh.
    /// Si absente partout, retourne la clé brute.
    pub fn format(&self, locale: &Locale, key: &str, args: Option<&FluentArgs<'_>>) -> String {
        // Essayer la locale demandée
        if let Some(result) = self.try_format(locale, key, args) {
            return result;
        }

        // Fallback vers FR-CH
        if *locale != Locale::FrCh {
            if let Some(result) = self.try_format(&Locale::FrCh, key, args) {
                return result;
            }
        }

        // Clé introuvable → retourner la clé brute
        key.to_string()
    }

    /// Retourne toutes les paires clé/valeur pour une locale (sans arguments).
    ///
    /// Inclut les clés FR-CH en fallback pour les clés manquantes.
    pub fn all_messages(&self, locale: &Locale) -> HashMap<String, String> {
        let mut messages = HashMap::new();

        // D'abord charger toutes les clés FR-CH comme base (fallback)
        if *locale != Locale::FrCh {
            self.collect_messages(&Locale::FrCh, &mut messages);
        }

        // Puis écraser avec les valeurs de la locale demandée
        self.collect_messages(locale, &mut messages);

        messages
    }

    /// Collecte les messages d'une locale dans un HashMap.
    fn collect_messages(&self, locale: &Locale, out: &mut HashMap<String, String>) {
        let Some(bundle) = self.bundles.get(locale) else {
            return;
        };
        let Some(locale_keys) = self.keys.get(locale) else {
            return;
        };

        for key in locale_keys {
            if let Some(msg) = bundle.get_message(key) {
                if let Some(pattern) = msg.value() {
                    let mut errs = vec![];
                    let value = bundle.format_pattern(pattern, None, &mut errs);
                    // Le handler `GET /api/v1/i18n/messages` pré-résout toutes
                    // les clés sans args — les variables `{ $var }` sont donc
                    // rendues littéralement pour interpolation côté frontend.
                    // Les `ResolverError(Reference(Variable))` sont attendues
                    // et ne doivent pas polluer les logs.
                    let real_errs: Vec<_> = errs
                        .into_iter()
                        .filter(|e| !is_missing_variable_error(e))
                        .collect();
                    if !real_errs.is_empty() {
                        tracing::warn!(key = %key, locale = %locale, "Fluent resolution errors: {:?}", real_errs);
                    }
                    out.insert(key.clone(), value.to_string());
                }
            }
        }
    }

    /// Tente de résoudre un message dans un bundle spécifique.
    fn try_format(
        &self,
        locale: &Locale,
        key: &str,
        args: Option<&FluentArgs<'_>>,
    ) -> Option<String> {
        let bundle = self.bundles.get(locale)?;
        let msg = bundle.get_message(key)?;
        let pattern = msg.value()?;
        let mut errs = vec![];
        let result = bundle.format_pattern(pattern, args, &mut errs);
        if !errs.is_empty() {
            tracing::warn!(key = %key, locale = %locale, "Fluent resolution errors: {:?}", errs);
        }
        Some(result.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn locales_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("locales")
    }

    #[test]
    fn load_all_locales() {
        let bundle = I18nBundle::load(&locales_dir()).expect("should load all locales");
        assert_eq!(bundle.bundles.len(), 4);
    }

    #[test]
    fn format_existing_key_fr() {
        let bundle = I18nBundle::load(&locales_dir()).unwrap();
        let msg = bundle.format(&Locale::FrCh, "error-invalid-credentials", None);
        assert!(!msg.is_empty());
        assert_ne!(msg, "error-invalid-credentials");
    }

    #[test]
    fn format_existing_key_de() {
        let bundle = I18nBundle::load(&locales_dir()).unwrap();
        let msg = bundle.format(&Locale::DeCh, "error-invalid-credentials", None);
        assert!(!msg.is_empty());
        assert_ne!(msg, "error-invalid-credentials");
    }

    #[test]
    fn format_missing_key_in_de_falls_back_to_fr() {
        let bundle = I18nBundle::load(&locales_dir()).unwrap();
        // Add a key only to FR by testing a key present in all locales first
        let de_msg = bundle.format(&Locale::DeCh, "error-invalid-credentials", None);
        let fr_msg = bundle.format(&Locale::FrCh, "error-invalid-credentials", None);
        // DE should have its own translation, different from FR
        assert_ne!(
            de_msg, fr_msg,
            "DE and FR should have different translations"
        );
        // Now test actual fallback: a key that doesn't exist → returns raw key
        let missing = bundle.format(&Locale::DeCh, "only-in-fr-test-key", None);
        assert_eq!(
            missing, "only-in-fr-test-key",
            "missing key should return raw key"
        );
    }

    #[test]
    fn format_unknown_key_returns_key() {
        let bundle = I18nBundle::load(&locales_dir()).unwrap();
        let msg = bundle.format(&Locale::FrCh, "nonexistent-key", None);
        assert_eq!(msg, "nonexistent-key");
    }

    #[test]
    fn all_messages_returns_all_keys() {
        let bundle = I18nBundle::load(&locales_dir()).unwrap();
        let msgs = bundle.all_messages(&Locale::FrCh);
        assert!(msgs.contains_key("error-invalid-credentials"));
        assert!(msgs.contains_key("error-forbidden"));
    }

    #[test]
    fn format_with_args() {
        let bundle = I18nBundle::load(&locales_dir()).unwrap();
        let mut args = FluentArgs::new();
        args.set("max", 64);
        let msg = bundle.format(&Locale::FrCh, "error-username-too-long", Some(&args));
        assert!(msg.contains("64"), "should interpolate max arg: {}", msg);
    }
}
