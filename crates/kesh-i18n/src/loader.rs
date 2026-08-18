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
        if *locale != Locale::FrCh
            && let Some(result) = self.try_format(&Locale::FrCh, key, args)
        {
            return result;
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
            if let Some(msg) = bundle.get_message(key)
                && let Some(pattern) = msg.value()
            {
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

    /// **AC8** (Story 16-3b, #151) — les deux clés du numéro de client sont
    /// traduites dans les **quatre** locales, et aucune ne retombe sur le
    /// français.
    ///
    /// ⚠️ **Le repli est SILENCIEUX, et c'est tout le problème.** `load()`
    /// charge d'abord toutes les clés `fr-CH` comme base de repli des autres
    /// locales : une clé seulement française rend le libellé **français sur un
    /// PDF allemand**, sans erreur, sans avertissement, avec tous les autres
    /// tests au vert. C'est le mécanisme exact de la KF #283 — 57 clés déjà
    /// manquantes en de-CH / it-CH / en-CH — appliqué ici à l'artefact qui donne
    /// son titre à la story.
    ///
    /// ⚠️ L'assertion de `types.rs` ne couvre pas ce cas : elle apparie
    /// `I18N_KEYS` et `DEFAULT_EN`, et **`DEFAULT_EN` n'est jamais atteint en
    /// production** — il ne sert que de repli de dernier recours côté crate.
    #[test]
    fn client_number_labels_are_translated_in_all_four_locales() {
        let bundle = I18nBundle::load(&locales_dir()).unwrap();
        // Celle du PDF, et celle de la fiche contact — AC8 porte sur les deux.
        for key in ["invoice-pdf-client-number", "contact-form-client-number"] {
            let fr = bundle.format(&Locale::FrCh, key, None);
            assert_ne!(fr, key, "{key} doit exister en fr-CH");
            for locale in [Locale::DeCh, Locale::ItCh, Locale::EnCh] {
                let msg = bundle.format(&locale, key, None);
                assert_ne!(msg, key, "{key} doit exister en {locale:?}");
                assert_ne!(
                    msg, fr,
                    "{key} en {locale:?} vaut le libellé FRANÇAIS — la clé est \
                     absente de cette locale et le loader replie en silence"
                );
            }
        }
    }

    /// **Story 22-2b (#301)** — les quatre libellés des sondes anti-doublon
    /// existent dans les quatre locales.
    ///
    /// ⚠️ L'assertion `!= fr` est la seule qui attrape le défaut réel : une clé
    /// absente d'une locale **retombe silencieusement sur le français**
    /// (`format_missing_key_in_de_falls_back_to_fr`), et `kesh-i18n` n'a
    /// **aucun test de parité globale** — les fichiers sont d'ailleurs déjà
    /// désappariés (KF #283). Un test qui vérifierait seulement « la clé rend
    /// autre chose que son nom » serait vert sur trois locales vides.
    ///
    /// ⚠️ `contact-duplicate-ide-archived` doit **dire autre chose** que
    /// `contact-duplicate-ide-active` : c'est ce libellé-là qui est le
    /// **recours** de l'utilisateur. Sans la mention « archivé », il reçoit un
    /// avertissement sur un contact introuvable dans son carnet, qu'il ne
    /// pourra ni modifier ni désarchiver, et dont il ne comprendra pas le `409`
    /// qui suit.
    #[test]
    fn duplicate_probe_labels_are_translated_in_all_four_locales() {
        let bundle = I18nBundle::load(&locales_dir()).unwrap();
        for key in [
            "contact-duplicate-heading",
            "contact-duplicate-others-count",
            "contact-duplicate-ide-active",
            "contact-duplicate-ide-archived",
        ] {
            let fr = bundle.format(&Locale::FrCh, key, None);
            assert_ne!(fr, key, "{key} doit exister en fr-CH");
            for locale in [Locale::DeCh, Locale::ItCh, Locale::EnCh] {
                let msg = bundle.format(&locale, key, None);
                assert_ne!(msg, key, "{key} doit exister en {locale:?}");
                assert_ne!(
                    msg, fr,
                    "{key} en {locale:?} vaut le libellé FRANÇAIS — la clé est \
                     absente de cette locale et le loader replie en silence"
                );
            }
        }

        // Le porteur ARCHIVÉ ne dit pas la même chose que le porteur actif :
        // c'est le libellé qui porte le recours.
        for locale in [Locale::FrCh, Locale::DeCh, Locale::ItCh, Locale::EnCh] {
            let actif = bundle.format(&locale, "contact-duplicate-ide-active", None);
            let archive = bundle.format(&locale, "contact-duplicate-ide-archived", None);
            assert_ne!(
                actif, archive,
                "en {locale:?}, l'avertissement du porteur archivé doit DIRE \
                 autre chose — c'est lui qui explique le 409 sans recours"
            );
        }
    }

    /// **Story 22-2b (#301)** — le libellé de recherche énumère l'IDE, **avec le
    /// sigle de sa propre locale**.
    ///
    /// ⚠️ **C'est le seul item de la story dont l'oubli partiel n'aurait fait
    /// rougir AUCUN gate.** `contact-filter-search-placeholder` est une clé
    /// **modifiée**, pas neuve : le test ci-dessus ne la voit pas, et mettre à
    /// jour `fr-CH` en oubliant les trois autres laisserait une valeur
    /// présente, différente de son nom, et différente du français — puisque ce
    /// sont des traductions distinctes.
    ///
    /// ⚠️ **Le sigle SUIT LA LOCALE** (`contact-col-ide` : `IDE` / `UID` / `IDI`
    /// / `UID`). Un test écrivant `contains("IDE")` sur les quatre échouerait
    /// sur trois — et le « corriger » en poussant `IDE` partout introduirait
    /// une régression de terminologie que rien d'autre ne rattraperait.
    ///
    /// ⚠️ Et le libellé dit **« sans séparateurs »** : la colonne est stockée
    /// normalisée (`CHE109322551`) tandis que le `LIKE` porte sur le terme
    /// brut, donc la forme imprimée sur une facture ne remonte **rien**.
    /// Promettre « ou IDE » tout court serait promettre le geste qui échoue.
    #[test]
    fn search_placeholder_lists_the_ide_with_each_locale_own_token() {
        let bundle = I18nBundle::load(&locales_dir()).unwrap();
        const KEY: &str = "contact-filter-search-placeholder";
        for (locale, jeton, reserve) in [
            (Locale::FrCh, "IDE", "sans séparateurs"),
            (Locale::DeCh, "UID", "ohne Trennzeichen"),
            (Locale::ItCh, "IDI", "senza separatori"),
            (Locale::EnCh, "UID", "without separators"),
        ] {
            let v = bundle.format(&locale, KEY, None);
            assert!(
                v.contains(jeton),
                "en {locale:?}, {KEY} n'énumère pas l'IDE sous son sigle local ({jeton}) : {v}"
            );
            assert!(
                v.contains(reserve),
                "en {locale:?}, {KEY} promet une recherche par IDE SANS dire « {reserve} » — \
                 or la forme imprimée sur une facture ne remonte rien : {v}"
            );
            // Le sigle annoncé est bien celui que le reste de l'interface emploie.
            assert_eq!(
                bundle.format(&locale, "contact-col-ide", None),
                jeton,
                "le sigle de {locale:?} a changé : ce test et le libellé doivent suivre"
            );
        }
    }

    /// **Story 22-4a (#167)** — le message d'« administration interdite via clé
    /// API » existe dans les quatre locales, et **dit autre chose** que celui de
    /// la gestion de clés.
    ///
    /// ⚠️ Les deux assertions sont indispensables, et pour deux raisons
    /// distinctes :
    ///
    /// - `!= fr` sur les trois autres locales, parce qu'une clé manquante
    ///   **retombe silencieusement sur le français** — un test qui vérifierait
    ///   seulement « la clé rend autre chose que son nom » serait vert sur trois
    ///   locales vides. `kesh-i18n` n'a aucun test de parité globale, et les
    ///   fichiers sont déjà désappariés (KF #283).
    /// - `!= management` parce que la décision D2 crée un code distinct **au
    ///   motif que réutiliser l'ancien message mentirait à l'appelant** : il
    ///   parle de gestion de clés. Or la tâche prescrit de *calquer* le bras
    ///   existant, et un calque qui copie le message satisferait toutes les
    ///   autres clauses de preuve.
    #[test]
    fn admin_forbidden_message_is_translated_and_distinct_from_key_management() {
        let bundle = I18nBundle::load(&locales_dir()).unwrap();
        const KEY: &str = "error-api-key-admin-forbidden";
        const NEIGHBOUR: &str = "error-api-key-management-forbidden";

        let fr = bundle.format(&Locale::FrCh, KEY, None);
        assert_ne!(fr, KEY, "{KEY} doit exister en fr-CH");

        for locale in [Locale::DeCh, Locale::ItCh, Locale::EnCh] {
            let msg = bundle.format(&locale, KEY, None);
            assert_ne!(msg, KEY, "{KEY} doit exister en {locale:?}");
            assert_ne!(
                msg, fr,
                "{KEY} en {locale:?} vaut le libellé FRANÇAIS — la clé est \
                 absente de cette locale et le loader replie en silence"
            );
        }

        for locale in [Locale::FrCh, Locale::DeCh, Locale::ItCh, Locale::EnCh] {
            assert_ne!(
                bundle.format(&locale, KEY, None),
                bundle.format(&locale, NEIGHBOUR, None),
                "en {locale:?}, le message d'administration interdite est \
                 IDENTIQUE à celui de la gestion de clés — c'est précisément le \
                 mensonge que la décision D2 refuse"
            );
        }
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
