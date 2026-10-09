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

    /// **Story 25-4-b2 (#416)** — les libellés du PDF de rappel et le refus du
    /// reste nul existent dans les quatre locales, et ne replient pas sur le
    /// français (même mécanisme que la KF #283, cf. le test précédent).
    #[test]
    fn reminder_pdf_labels_are_translated_in_all_four_locales() {
        let bundle = I18nBundle::load(&locales_dir()).unwrap();
        for key in [
            "invoice-pdf-reminder-title",
            "invoice-pdf-settled",
            "invoice-pdf-amount-due",
            "invoice-pdf-reminder-fees",
            "invoice-pdf-reminder-fees-note",
            "error-reminder-nothing-due",
            "error-reminder-amounts-changed",
            "reminders-error-nothing-due",
        ] {
            let fr = bundle.format(&Locale::FrCh, key, None);
            assert_ne!(fr, key, "{key} doit exister en fr-CH");
            for locale in [Locale::DeCh, Locale::ItCh, Locale::EnCh] {
                let msg = bundle.format(&locale, key, None);
                assert_ne!(msg, key, "{key} doit exister en {locale:?}");
                assert_ne!(msg, fr, "{key} en {locale:?} replie sur le français");
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
            // ⚠️ Ajoutée en passe 4. Elle était la SEULE des cinq clés du
            // dispositif à échapper à l'assertion `!= fr` — c'est-à-dire au seul
            // contrôle qui attrape le repli silencieux sur le français. Le test
            // du chemin réel ne bouchait pas le trou : « et 1 autre » satisfait
            // ses assertions aussi bien en allemand qu'en français.
            "contact-duplicate-others-count-one",
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

    /// Les clés à **sélecteur Fluent** que le dépôt s'autorise, et pour lesquelles
    /// quelqu'un a vérifié qu'elles sont résolues **côté serveur, avec arguments**.
    ///
    /// `contact-payment-terms-days-label` l'est par `routes/contacts.rs`, via
    /// `i18n.format(&locale, clé, Some(&args))`.
    ///
    /// `error-account-not-postable` (Story 15-5a, choix C18) l'est par le bras
    /// `DbError::AccountsNotPostable` de `kesh-api/src/errors.rs`, via `t_args`
    /// avec `numbers` et `count` (nombre Fluent). C'est sûr : le frontend affiche
    /// `err.message`, déjà résolu par le serveur, et ne lit jamais cette clé dans
    /// son dictionnaire.
    ///
    /// `error-designated-account-not-postable` (Story 15-5d, choix C28/C36) l'est
    /// par le bras `DbError::DesignatedAccountsNotPostable`, par le même helper
    /// (`account_not_postable_response`) et les mêmes arguments.
    const SELECTEURS_RESOLUS_COTE_SERVEUR: &[&str] = &[
        "contact-payment-terms-days-label",
        "error-account-not-postable",
        "error-designated-account-not-postable",
    ];

    /// Les clés d'un `.ftl` dont la valeur contient une expression **`select`**.
    ///
    /// ⚠️ **Trois formes de Fluent sont valides, et la détection doit les couvrir
    /// TOUTES** — une version antérieure n'en voyait qu'une, celle où le `{`
    /// suit immédiatement le `=`. Les deux autres passaient au travers :
    ///
    /// ```text
    /// a = { $n -> …        ← la forme canonique, seule détectée alors
    /// b = et { $n -> …     ← du texte AVANT le sélecteur
    /// c =
    ///     { $n -> …        ← la valeur commence à la ligne suivante
    /// ```
    ///
    /// D'où un balayage du **fichier entier** plutôt que ligne à ligne : on
    /// retient la dernière clé rencontrée, et tout `->` non commenté **précédé
    /// d'une accolade ouvrante sur la même ligne** la désigne.
    /// La fonction est **pure** pour que les trois formes s'éprouvent sur des
    /// chaînes littérales, sans jamais écrire dans le dépôt.
    fn cles_a_selecteur(ftl: &str) -> Vec<String> {
        let mut trouves: Vec<String> = Vec::new();
        let mut courante: Option<String> = None;
        for ligne in ftl.lines() {
            let nu = ligne.trim_start();
            if nu.starts_with('#') {
                continue;
            }
            // Une ligne NON indentée qui porte `=` ouvre un nouveau message.
            if !ligne.starts_with(char::is_whitespace)
                && let Some((cle, _)) = ligne.split_once('=')
                && !cle.trim().is_empty()
            {
                courante = Some(cle.trim().to_string());
            }
            // ⚠️ Un `->` ne désigne un sélecteur que s'il suit une **accolade
            // ouvrante sur la même ligne**. Sans cette exigence, une flèche
            // écrite en toutes lettres dans un libellé — `Cliquez sur -> pour
            // continuer` — était classée sélecteur, et le garde-fou aurait
            // rougi sur une clé parfaitement innocente. Le coût de ce faux
            // positif n'est pas le rouge lui-même : c'est l'ajout réflexe de la
            // clé à `SELECTEURS_RESOLUS_COTE_SERVEUR` pour faire taire un test
            // incompris — ce qui désarmerait le garde-fou pour de bon.
            // Relevé en passe 5 de revue de code.
            if let Some(pos) = ligne.find("->")
                && ligne[..pos].contains('{')
                && let Some(cle) = &courante
                && !trouves.contains(cle)
            {
                trouves.push(cle.clone());
            }
        }
        trouves
    }

    /// **Les trois formes de sélecteur sont détectées — éprouvé, pas supposé.**
    ///
    /// ⚠️ Le garde-fou avait été déclaré « mis en défaut » sur la seule forme
    /// canonique. C'est la faute que ce dossier collectionne : éprouver un cas,
    /// et énoncer la règle pour tous.
    #[test]
    fn the_select_detector_catches_all_three_fluent_forms() {
        let canonique = "a = { $n ->\n    [one] x\n   *[other] y\n}\n";
        let texte_avant = "b = et { $n ->\n    [one] x\n   *[other] y\n}\n";
        let valeur_a_la_ligne = "c =\n    { $n ->\n        [one] x\n       *[other] y\n    }\n";
        let sans_selecteur = "d = juste du texte\ne = avec { $var } interpolée\n";
        // ⚠️ Contre-exemple ajouté en passe 5 : une FLÈCHE ÉCRITE EN TOUTES
        // LETTRES dans un libellé. Aucune locale n'en contient aujourd'hui —
        // c'est donc un défaut latent, que le premier traducteur écrivant
        // « cliquez sur -> » aurait réveillé.
        let fleche_litterale = "g = Cliquez sur -> pour continuer\n";
        let commentaire = "# f = { $n -> [one] x *[other] y }\n";

        assert_eq!(cles_a_selecteur(canonique), vec!["a"], "forme canonique");
        assert_eq!(
            cles_a_selecteur(texte_avant),
            vec!["b"],
            "texte AVANT le sélecteur"
        );
        assert_eq!(
            cles_a_selecteur(valeur_a_la_ligne),
            vec!["c"],
            "valeur commençant à la ligne suivante"
        );
        assert!(
            cles_a_selecteur(sans_selecteur).is_empty(),
            "aucun faux positif"
        );
        assert!(
            cles_a_selecteur(commentaire).is_empty(),
            "un commentaire ne compte pas"
        );
        assert!(
            cles_a_selecteur(fleche_litterale).is_empty(),
            "une flèche en toutes lettres n'est pas un sélecteur"
        );
    }

    /// **Aucun sélecteur ne doit être servi par le dictionnaire du frontend.**
    ///
    /// ⚠️ Ce garde-fou existe parce que le piège s'est refermé une fois, et qu'il
    /// est **entièrement muet** : `all_messages` formate chaque message une seule
    /// fois et **sans arguments**, si bien qu'une expression `{ $x -> … }` y est
    /// résolue à l'aveugle sur sa branche `*[other]`, qui se retrouve figée dans
    /// le dictionnaire. Rien n'échoue, rien ne prévient — l'interface affiche
    /// simplement toujours le pluriel.
    ///
    /// **Un test qui appelle `format()` avec des arguments ne voit RIEN de tout
    /// cela** : il emprunte un chemin que le frontend ne prend jamais. C'est
    /// exactement ce qui s'est produit — un correctif de pluriel déclaré prouvé,
    /// inerte en production, et vert au gate.
    ///
    /// Ajouter un sélecteur oblige donc à l'inscrire ci-dessus, c'est-à-dire à
    /// dire OÙ il est résolu avec ses arguments.
    #[test]
    fn no_new_select_expression_reaches_the_frontend_dictionary() {
        let mut trouves: Vec<String> = Vec::new();
        for locale in ["fr-CH", "de-CH", "it-CH", "en-CH"] {
            let chemin = locales_dir().join(locale).join("messages.ftl");
            let texte = std::fs::read_to_string(&chemin)
                .unwrap_or_else(|e| panic!("lecture de {chemin:?} : {e}"));
            for cle in cles_a_selecteur(&texte) {
                if !trouves.contains(&cle) {
                    trouves.push(cle);
                }
            }
        }
        trouves.sort();

        let mut attendus: Vec<String> = SELECTEURS_RESOLUS_COTE_SERVEUR
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        attendus.sort();

        assert_eq!(
            trouves, attendus,
            "\nUn sélecteur Fluent a été ajouté ou retiré.\n\n\
             Si c'est un AJOUT : le dictionnaire servi au frontend le résoudra \
             SANS arguments, donc toujours sur sa branche `*[other]`, EN SILENCE. \
             Ou bien la clé est résolue côté serveur avec ses arguments — et il \
             faut alors l'inscrire dans SELECTEURS_RESOLUS_COTE_SERVEUR en disant \
             où —, ou bien il faut la scinder en clés plates.\n"
        );

        // Et la DÉMONSTRATION du mécanisme, pour que la règle ci-dessus ne soit
        // pas du folklore : la clé autorisée ressort bien amputée du dictionnaire.
        let bundle = I18nBundle::load(&locales_dir()).unwrap();
        let dico = bundle.all_messages(&Locale::FrCh);
        let servi = dico
            .get("contact-payment-terms-days-label")
            .expect("la clé doit exister");
        assert!(
            !servi.contains("->"),
            "le dictionnaire rend une branche unique, pas le sélecteur : {servi}"
        );
    }

    /// **Le compteur « et N autres » s'accorde en nombre — par DEUX CLÉS, pas par
    /// un sélecteur Fluent.**
    ///
    /// ⚠️ **Ce test a été réécrit après avoir prouvé quelque chose de faux.** Sa
    /// première version appelait `bundle.format(clé, Some(args))` sur une clé
    /// portant un sélecteur `{ $count -> [one] … *[other] … }`, et il passait.
    /// Mais **le frontend n'emprunte jamais ce chemin** : il reçoit un
    /// dictionnaire pré-résolu par `all_messages`, qui formate chaque message
    /// UNE SEULE FOIS et SANS arguments. Sur une expression `select`, Fluent doit
    /// alors choisir une branche à l'aveugle — il prend `*[other]` et la fige.
    /// L'application affichait donc « et 1 autres », c'est-à-dire exactement le
    /// défaut que le correctif prétendait fermer.
    ///
    /// D'où deux clés plates, et un test qui interroge **`all_messages`**, la
    /// seule porte que le frontend franchit réellement.
    #[test]
    fn the_others_count_agrees_in_number_through_the_real_path() {
        let bundle = I18nBundle::load(&locales_dir()).unwrap();
        const UN: &str = "contact-duplicate-others-count-one";
        const N: &str = "contact-duplicate-others-count";

        for locale in [Locale::FrCh, Locale::DeCh, Locale::ItCh, Locale::EnCh] {
            let dico = bundle.all_messages(&locale);

            let un = dico.get(UN).unwrap_or_else(|| {
                panic!("en {locale:?}, {UN} est absente du DICTIONNAIRE servi au frontend")
            });
            let n = dico.get(N).unwrap_or_else(|| {
                panic!("en {locale:?}, {N} est absente du DICTIONNAIRE servi au frontend")
            });

            assert_ne!(un, n, "en {locale:?}, les deux formes sont identiques");

            // La forme SINGULIÈRE porte le nombre en toutes lettres et n'attend
            // aucun argument : c'est ce qui la rend servable par le dictionnaire.
            assert!(
                un.contains('1'),
                "en {locale:?}, la forme singulière perd le nombre : {un}"
            );
            assert!(
                !un.contains("$count"),
                "en {locale:?}, la forme singulière attend un argument — le \
                 dictionnaire ne peut pas la résoudre : {un}"
            );

            // La forme PLURIELLE, elle, garde son placeholder : `i18nMsg` le
            // substitue côté client.
            assert!(
                n.contains("$count"),
                "en {locale:?}, la forme plurielle a perdu son placeholder : {n}"
            );
        }

        // Le cas français, celui qui a motivé le correctif, épinglé nommément.
        let fr = bundle.all_messages(&Locale::FrCh);
        assert_eq!(fr.get(UN).map(String::as_str), Some("et 1 autre"));
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

    /// **Story 23-1a (#283) — GARDE A : les quatre catalogues déclarent le MÊME
    /// ensemble de clés.**
    ///
    /// ⚠️ **Ce test compare des ENSEMBLES DE CLÉS, jamais des textes rendus, et c'est
    /// la seule façon honnête de le faire.** `format()` et `all_messages()` replient
    /// sur `fr-CH` (cf. `all_messages`, plus haut) : pour une clé absente d'une locale
    /// ils rendent **le texte français**, exactement comme si la traduction existait.
    /// Un test bâti dessus ne peut pas distinguer « traduit » de « replié ».
    ///
    /// ⚠️ **Et l'astuce `assert_ne!(msg, fr)` des deux gardes bornées voisines ne se
    /// généralise pas** : « Total », « CHF », « Journal », « Kesh » sont légitimement
    /// identiques en quatre langues. Elle vaut pour un domaine choisi, pas pour 1273 clés.
    ///
    /// L'allowlist `dette-parite-connue.txt` porte les 57 clés de l'issue #283, et le
    /// test échoue **dans les deux sens** : clé manquante non listée, ou entrée de la
    /// liste devenue présente / disparue de `fr-CH`.
    #[test]
    fn parity_between_locales() {
        let bundle = I18nBundle::load(&locales_dir()).unwrap();

        let cles = |l: Locale| -> std::collections::BTreeSet<String> {
            bundle.keys.get(&l).unwrap().iter().cloned().collect()
        };
        let fr = cles(Locale::FrCh);
        let cibles = [Locale::DeCh, Locale::ItCh, Locale::EnCh];

        // Borne anti-test-muet : un chargement cassé rendrait des ensembles vides, et
        // toutes les comparaisons ci-dessous seraient vertes.
        for locale in Locale::ALL {
            assert!(
                cles(locale).len() >= 1200,
                "{locale:?} ne déclare que {} clés — le chargement est cassé, pas le catalogue",
                cles(locale).len()
            );
        }

        let dette: std::collections::BTreeSet<String> = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/dette-parite-connue.txt"
        ))
        .expect("dette-parite-connue.txt introuvable")
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_string)
        .collect();

        // (1) Toute clé de fr-CH absente d'une locale cible doit être dans la dette.
        let mut manquantes = Vec::new();
        for locale in cibles {
            let ici = cles(locale);
            for k in fr.difference(&ici) {
                if !dette.contains(k) {
                    manquantes.push(format!("{k} absente de {locale:?}"));
                }
            }
        }

        // (2) Une clé présente SEULEMENT dans une locale cible est un défaut symétrique.
        for locale in cibles {
            for k in cles(locale).difference(&fr) {
                manquantes.push(format!("{k} présente en {locale:?} mais absente de fr-CH"));
            }
        }

        // (3) L'allowlist ne se fossilise pas : une entrée résolue ou disparue la fait
        //     échouer. « Au moins une » des trois cibles, et non « les quatre » — sans
        //     quoi une traduction partielle (de-CH et it-CH mais pas en-CH) resterait
        //     invisible des deux contrôles.
        let mut obsoletes = Vec::new();
        for k in &dette {
            if !fr.contains(k) {
                obsoletes.push(format!("{k} : plus dans fr-CH — retirer de la dette"));
            } else if cibles.iter().any(|l| cles(*l).contains(k)) {
                obsoletes.push(format!("{k} : désormais traduite — retirer de la dette"));
            }
        }

        assert!(
            manquantes.is_empty() && obsoletes.is_empty(),
            "parité des catalogues rompue.\n  Clés manquantes hors dette ({}) :\n    {}\n  \
             Entrées de dette obsolètes ({}) :\n    {}",
            manquantes.len(),
            manquantes.join("\n    "),
            obsoletes.len(),
            obsoletes.join("\n    ")
        );
    }

    /// Story 15-5a (AC2) — la clé du refus « compte non imputable » se résout,
    /// **avec arguments**, au singulier et au pluriel dans les quatre locales,
    /// et nomme les numéros passés. Testée par `format` avec arguments : le
    /// dictionnaire du frontend (`all_messages`) rendrait toujours `*[other]`.
    #[test]
    fn account_not_postable_resolves_singular_and_plural_in_every_locale() {
        let bundle = I18nBundle::load(&locales_dir()).unwrap();
        const KEY: &str = "error-account-not-postable";
        for locale in Locale::ALL {
            let mut one = FluentArgs::new();
            one.set("numbers", "1000");
            one.set("count", 1usize);
            let singulier = bundle
                .format(&locale, KEY, Some(&one))
                .replace(['\u{2068}', '\u{2069}'], "");

            let mut many = FluentArgs::new();
            many.set("numbers", "1000, 2000");
            many.set("count", 2usize);
            let pluriel = bundle
                .format(&locale, KEY, Some(&many))
                .replace(['\u{2068}', '\u{2069}'], "");

            assert_ne!(singulier, KEY, "{locale:?} : clé absente");
            assert!(singulier.contains("1000"), "{locale:?} : {singulier}");
            assert!(pluriel.contains("1000, 2000"), "{locale:?} : {pluriel}");
            assert_ne!(
                singulier,
                pluriel.replace("1000, 2000", "1000"),
                "{locale:?} : le singulier doit différer du pluriel — sélecteur inopérant"
            );
            let bas = format!("{singulier} {pluriel}").to_lowercase();
            for interdit in ["archiv", "invalid", "ungültig", "non valid"] {
                assert!(
                    !bas.contains(interdit),
                    "{locale:?} : « {interdit} » dans {bas}"
                );
            }
        }
        // Le texte FR exact du singulier (AC2).
        let mut one = FluentArgs::new();
        one.set("numbers", "1000");
        one.set("count", 1usize);
        assert_eq!(
            bundle
                .format(&Locale::FrCh, KEY, Some(&one))
                .replace(['\u{2068}', '\u{2069}'], ""),
            "Le compte 1000 n’est pas imputable (compte de regroupement, de résultat ou de clôture) : choisissez un compte imputable."
        );
    }

    /// Story 15-5d (AC2) — la clé du refus « compte DÉSIGNÉ non imputable » se
    /// résout, avec arguments, au singulier et au pluriel dans les quatre
    /// locales ; elle nomme les numéros, renvoie aux réglages de facturation et
    /// dit qu'un administrateur doit agir (choix C36).
    #[test]
    fn designated_account_not_postable_resolves_singular_and_plural_in_every_locale() {
        let bundle = I18nBundle::load(&locales_dir()).unwrap();
        const KEY: &str = "error-designated-account-not-postable";
        for locale in Locale::ALL {
            let mut one = FluentArgs::new();
            one.set("numbers", "1100");
            one.set("count", 1usize);
            let singulier = bundle
                .format(&locale, KEY, Some(&one))
                .replace(['\u{2068}', '\u{2069}'], "");

            let mut many = FluentArgs::new();
            many.set("numbers", "1100, 2000");
            many.set("count", 2usize);
            let pluriel = bundle
                .format(&locale, KEY, Some(&many))
                .replace(['\u{2068}', '\u{2069}'], "");

            assert_ne!(singulier, KEY, "{locale:?} : clé absente");
            assert!(singulier.contains("1100"), "{locale:?} : {singulier}");
            assert!(pluriel.contains("1100, 2000"), "{locale:?} : {pluriel}");
            assert_ne!(
                singulier,
                pluriel.replace("1100, 2000", "1100"),
                "{locale:?} : le singulier doit différer du pluriel — sélecteur inopérant"
            );
            // Où agir : le menu tel que la locale l'affiche.
            let menu = match locale {
                Locale::FrCh => "Paramètres → Facturation",
                Locale::DeCh => "Einstellungen → Fakturierung",
                Locale::ItCh => "Impostazioni → Fatturazione",
                Locale::EnCh => "Settings → Invoicing",
            };
            for texte in [&singulier, &pluriel] {
                assert!(
                    texte.contains(menu),
                    "{locale:?} : « {menu} » absent de {texte}"
                );
            }
            // Qui agit (C36).
            let admin = format!("{singulier} {pluriel}").to_lowercase();
            assert!(
                ["administrat", "amministrat"]
                    .iter()
                    .any(|m| admin.contains(m)),
                "{locale:?} : l'administrateur n'est pas nommé : {admin}"
            );
            for interdit in ["archiv", "invalid", "ungültig", "non valid", "sous-compte"] {
                assert!(
                    !admin.contains(interdit),
                    "{locale:?} : « {interdit} » dans {admin}"
                );
            }
        }
        // Le texte FR exact (AC2), au singulier et au pluriel.
        let mut one = FluentArgs::new();
        one.set("numbers", "1100");
        one.set("count", 1usize);
        assert_eq!(
            bundle
                .format(&Locale::FrCh, KEY, Some(&one))
                .replace(['\u{2068}', '\u{2069}'], ""),
            "Le compte 1100, désigné dans Paramètres → Facturation, n’est pas imputable (compte de regroupement, de résultat ou de clôture) : un administrateur doit y désigner à sa place un compte imputable."
        );
        let mut many = FluentArgs::new();
        many.set("numbers", "1100, 2000");
        many.set("count", 2usize);
        assert_eq!(
            bundle
                .format(&Locale::FrCh, KEY, Some(&many))
                .replace(['\u{2068}', '\u{2069}'], ""),
            "Les comptes 1100, 2000, désignés dans Paramètres → Facturation, ne sont pas imputables (comptes de regroupement, de résultat ou de clôture) : un administrateur doit y désigner à leur place des comptes imputables."
        );
    }

    // ── Story 15-14a — gardes de texte des catalogues (G3, G8, G10, G11) ──────────

    /// Valeurs **brutes** d'un catalogue : `clé = valeur`, commentaires ignorés.
    ///
    /// Règle commune aux trois analyseurs de catalogue (`catalogue_fr` de
    /// `kesh-api/tests/textes_coherents.rs`, `valeurDuCatalogueFr` du Vitest G13 ; revue
    /// de code P2, B2-4, E2-4, A-4) : une ligne qui commence par un blanc prolonge la
    /// valeur (jointe par une espace, sauf à une valeur encore vide — forme bloc) ; toute
    /// autre ligne — tête, commentaire, ligne vide, `}` de sélecteur en colonne 0 — la
    /// clôt sans s'y ajouter. Avant la P2, celui-ci ajoutait aussi les lignes non
    /// indentées, dont le `}` ; anti-test-muet :
    /// `valeurs_brutes_suit_la_regle_commune_des_trois_analyseurs`.
    ///
    /// ⚠️ Volontairement **sans** repli sur `fr-CH` ni formatage Fluent : une clé
    /// absente d'une locale doit rougir, pas retomber silencieusement sur le français
    /// (c'est le défaut que `all_messages` masquerait).
    fn valeurs_brutes(locale: &str) -> HashMap<String, String> {
        let chemin = locales_dir().join(locale).join("messages.ftl");
        let texte = std::fs::read_to_string(&chemin)
            .unwrap_or_else(|e| panic!("lecture de {}: {e}", chemin.display()));
        let tete = regex::Regex::new(r"^([a-zA-Z][\w-]*) = ?(.*)$").unwrap();
        let mut out = HashMap::new();
        let mut courante: Option<String> = None;
        for ligne in texte.lines() {
            if let Some(c) = tete.captures(ligne) {
                let cle = c[1].to_string();
                out.insert(cle.clone(), c[2].to_string());
                courante = Some(cle);
            } else if ligne.starts_with([' ', '\t']) && !ligne.trim().is_empty() {
                if let Some(cle) = &courante {
                    let v: &mut String = out.get_mut(cle).unwrap();
                    if !v.is_empty() {
                        v.push(' ');
                    }
                    v.push_str(ligne.trim());
                }
            } else {
                courante = None;
            }
        }
        assert!(
            out.len() > 100,
            "{locale} : catalogue lu à vide ({})",
            out.len()
        );
        out
    }

    /// Anti-test-muet de [`valeurs_brutes`] — les trois cas réels de la règle commune
    /// (même test que `le_catalogue_fr_joint_les_continuations` côté `kesh-api` et que
    /// le Vitest G13).
    #[test]
    fn valeurs_brutes_suit_la_regle_commune_des_trois_analyseurs() {
        let fr = valeurs_brutes("fr-CH");
        let corps = &fr["email-password-reset-body"];
        assert!(
            corps.starts_with("Vous avez demandé") && corps.ends_with("ignorez cet email."),
            "continuations non jointes, ou blanc de tête : {corps:?}"
        );
        assert_eq!(fr["auth-recovery-forgot-title"], "Mot de passe oublié");
        let selecteur = &fr["error-account-not-postable"];
        assert!(
            selecteur.ends_with("choisissez des comptes imputables."),
            "`}}` de sélecteur ajouté, ou variante perdue : {selecteur:?}"
        );
    }

    const LOCALES: [&str; 4] = ["fr-CH", "de-CH", "it-CH", "en-CH"];

    /// **G3** (Story 15-14a, #547) — le message du compte de produit inutilisable
    /// nomme l'écran réel, dans chaque locale : son titre (`settings-invoicing-title`,
    /// « Paramètres — Facturation ») écrit comme un chemin (` — ` → ` → `). Et le
    /// message d'en-tête de PDF trop haut renvoie aux « Paramètres », non aux
    /// « réglages » (nom commun) en fr-CH.
    #[test]
    fn le_message_du_compte_de_produit_renvoie_a_l_ecran_reel() {
        for locale in LOCALES {
            let v = valeurs_brutes(locale);
            let titre = v
                .get("settings-invoicing-title")
                .unwrap_or_else(|| panic!("{locale} : settings-invoicing-title absent"));
            let chemin = titre.replace(" — ", " → ");
            assert_ne!(&chemin, titre, "{locale} : titre sans « — » : {titre}");
            let message = v
                .get("invoice-default-revenue-account-unusable")
                .unwrap_or_else(|| panic!("{locale} : clé absente"));
            assert!(
                message.contains(&chemin),
                "{locale} : « {chemin} » absent de : {message}"
            );
        }
        let fr = valeurs_brutes("fr-CH");
        let entete = &fr["error-invoice-pdf-header-overflow"];
        assert!(entete.contains("dans les Paramètres"), "{entete}");
        assert!(!entete.contains("dans les réglages"), "{entete}");
    }

    /// **G8** (Story 15-14a, #569, C-15-14-14) — toute clé qui **prescrit** une
    /// réouverture d'exercice dit l'ordre (« en commençant par le plus récent »),
    /// dans les quatre locales.
    ///
    /// Le domaine se prend en **fr-CH**, où le verbe est univoque (`rouvr`, `réouv`) ;
    /// en allemand il a au moins cinq formes, et un motif par locale passerait à vide
    /// sur l'une d'elles. Les exemptions sont **par clé**, liste fermée, et chacune
    /// doit encore appartenir au domaine — sans quoi l'exemption est morte.
    #[test]
    fn les_prescriptions_de_reouverture_disent_l_ordre() {
        const MARQUEURS: [(&str, &str); 4] = [
            ("fr-CH", "en commençant par le plus récent"),
            ("de-CH", "beginnend mit dem neuesten"),
            ("it-CH", "cominciando dal più recente"),
            ("en-CH", "starting with the most recent"),
        ];
        // Nomment l'acte, le décrivent, nomment l'exercice à rouvrir, ou autre objet.
        const EXEMPTEES: [&str; 10] = [
            "fiscal-year-reopen-button",
            "fiscal-year-reopen-confirmation-title",
            "fiscal-year-reopen-confirmation-action",
            "fiscal-year-reopen-motif-label",
            "error-fiscal-year-reopen-motif-empty",
            "error-fiscal-year-reopen-motif-too-long",
            "fiscal-year-close-confirmation-body",
            "fiscal-year-reopen-confirmation-body",
            "fiscal-year-reopen-blocked-later-closed",
            "error-reminder-amounts-changed",
        ];
        // Les six clés de #569 (anti-test-muet : nommées, elles doivent être au domaine).
        const CLES_569: [&str; 6] = [
            "settlement-cancel-blocked-fiscal-year-closed",
            "reconciliation-cancel-blocked-fiscal-year-closed",
            "supplier-invoices-cancel-blocked-fiscal-year-closed",
            "error-fiscal-year-reopen-blocked",
            "opening-balances-locked-first-year-closed",
            "error-opening-balances-first-year-closed",
        ];
        let verbe = regex::Regex::new(r"[Rr]ouvr|[Rr]éouv").unwrap();
        let catalogues: HashMap<&str, HashMap<String, String>> =
            LOCALES.iter().map(|l| (*l, valeurs_brutes(l))).collect();
        let mut domaine: Vec<&String> = catalogues["fr-CH"]
            .iter()
            .filter(|(_, v)| verbe.is_match(v))
            .map(|(k, _)| k)
            .collect();
        domaine.sort();
        for cle in EXEMPTEES.iter().chain(CLES_569.iter()) {
            assert!(
                domaine.iter().any(|d| d.as_str() == *cle),
                "{cle} n'est plus au domaine (sa valeur fr-CH ne prescrit plus de réouverture)"
            );
        }
        let mut controlees = 0;
        for cle in domaine.iter().filter(|c| !EXEMPTEES.contains(&c.as_str())) {
            for (locale, marqueur) in MARQUEURS {
                let valeur = catalogues[locale]
                    .get(cle.as_str())
                    .unwrap_or_else(|| panic!("{locale} : {cle} absente"));
                assert!(
                    valeur.contains(marqueur),
                    "{locale} : {cle} prescrit la réouverture sans l'ordre « {marqueur} » : {valeur}"
                );
            }
            controlees += 1;
        }
        // 6 clés de #569 + 4 qui portaient déjà le marqueur (C-15-14-14).
        assert!(
            controlees >= 10,
            "domaine contrôlé trop petit : {controlees}"
        );
    }

    /// **G8-bis** (Story 15-14a, revue de code P2, A-1, A-2, E2-2, B2-1) — toute
    /// prescription de réouverture est **bornée**, dans les quatre locales : elle dit
    /// jusqu'où rouvrir.
    ///
    /// La garde de création et la garde de réouverture (`FIND_LATER_CLOSED_SQL`,
    /// `start_date > ? AND status = 'Closed'`, `kesh-db/src/repositories/fiscal_years.rs`)
    /// ne bloquent que sur un exercice clôturé **postérieur** : « rouvrez les exercices
    /// clôturés » sans borne ferait rouvrir aussi les exercices antérieurs, et lever sans
    /// raison le verrou CO 957-964 de comptes arrêtés. Deux bornes justes coexistent :
    /// « les exercices **postérieurs** clôturés » (famille `LATER_FISCAL_YEAR_CLOSED` et
    /// réouverture refusée) et « **jusqu'à celui-ci** » (famille #569 : annulations,
    /// soldes de départ).
    ///
    /// Inventaire des non-bornées, non liste de formes : le domaine est celui de G8
    /// (verbe fr-CH), moins ses exemptions ; chaque clé y porte l'une des deux bornes
    /// dans chaque locale, ou la garde rougit en la nommant. S'y ajoute une assertion
    /// **positive** par clé et par locale sur la borne « postérieurs » des cinq clés
    /// qui la portent — sans quoi un retour à la formule large sur l'une d'elles, avec
    /// « jusqu'à » ailleurs dans la phrase, passerait.
    #[test]
    fn les_prescriptions_de_reouverture_sont_bornees() {
        const POSTERIEURS: [(&str, &str); 4] = [
            ("fr-CH", "les exercices postérieurs clôturés"),
            ("de-CH", "die späteren abgeschlossenen Geschäftsjahre"),
            ("it-CH", "gli esercizi successivi chiusi"),
            ("en-CH", "the later closed fiscal years"),
        ];
        const JUSQU_A: [(&str, &[&str]); 4] = [
            ("fr-CH", &["jusqu'à celui-ci", "jusqu’à celui-ci"]),
            ("de-CH", &["bis zu diesem Geschäftsjahr"]),
            ("it-CH", &["fino a questo esercizio"]),
            ("en-CH", &["down to this one"]),
        ];
        const CLES_POSTERIEURS: [&str; 5] = [
            "error-fiscal-year-reopen-blocked",
            "error-fiscal-year-create-later-closed",
            "error-later-fiscal-year-closed",
            "journal-entries-modify-blocked-later-fiscal-year-closed",
            "fiscal-year-out-of-order-warning",
        ];
        // Même domaine et mêmes exemptions que G8 (`les_prescriptions_de_reouverture_disent_l_ordre`).
        const EXEMPTEES: [&str; 10] = [
            "fiscal-year-reopen-button",
            "fiscal-year-reopen-confirmation-title",
            "fiscal-year-reopen-confirmation-action",
            "fiscal-year-reopen-motif-label",
            "error-fiscal-year-reopen-motif-empty",
            "error-fiscal-year-reopen-motif-too-long",
            "fiscal-year-close-confirmation-body",
            "fiscal-year-reopen-confirmation-body",
            "fiscal-year-reopen-blocked-later-closed",
            "error-reminder-amounts-changed",
        ];
        let verbe = regex::Regex::new(r"[Rr]ouvr|[Rr]éouv").unwrap();
        let catalogues: HashMap<&str, HashMap<String, String>> =
            LOCALES.iter().map(|l| (*l, valeurs_brutes(l))).collect();
        let mut domaine: Vec<&String> = catalogues["fr-CH"]
            .iter()
            .filter(|(k, v)| verbe.is_match(v) && !EXEMPTEES.contains(&k.as_str()))
            .map(|(k, _)| k)
            .collect();
        domaine.sort();
        for cle in &CLES_POSTERIEURS {
            assert!(
                domaine.iter().any(|d| d.as_str() == *cle),
                "{cle} n'est plus au domaine"
            );
            for (locale, borne) in POSTERIEURS {
                let valeur = &catalogues[locale][*cle];
                assert!(
                    valeur.contains(borne),
                    "{locale} : {cle} sans « {borne} » : {valeur}"
                );
            }
        }
        let mut bornees = 0;
        for cle in &domaine {
            for ((locale, posterieurs), (_, jusqu_a)) in POSTERIEURS.iter().zip(JUSQU_A) {
                let valeur = catalogues[locale]
                    .get(cle.as_str())
                    .unwrap_or_else(|| panic!("{locale} : {cle} absente"));
                assert!(
                    valeur.contains(posterieurs) || jusqu_a.iter().any(|j| valeur.contains(j)),
                    "{locale} : {cle} prescrit une réouverture sans borne : {valeur}"
                );
            }
            bornees += 1;
        }
        // 5 clés « postérieurs » + 5 clés « jusqu'à celui-ci » (#569) ; une clé neuve
        // du domaine s'y ajoute et doit être bornée, d'où un plancher et non un compte.
        assert!(bornees >= 10, "domaine contrôlé trop petit : {domaine:?}");
    }

    /// **G10** (Story 15-14a, #321) — l'allemand de Suisse écrit `MWST`, jamais `MwSt`.
    #[test]
    fn glossaire_mwst() {
        let de = valeurs_brutes("de-CH");
        let mwst = regex::Regex::new(r"\bMwSt\b").unwrap();
        let fautives: Vec<_> = de.iter().filter(|(_, v)| mwst.is_match(v)).collect();
        assert!(fautives.is_empty(), "MwSt en de-CH : {fautives:?}");
        assert!(
            de.values().any(|v| v.contains("MWST")),
            "aucune valeur de-CH ne porte MWST (lecture à vide ?)"
        );
    }

    /// **G11** (Story 15-14a, #323, C-15-14-7, C-15-14-18) — clôturer un exercice
    /// n'emprunte pas le verbe qui ferme un panneau.
    #[test]
    fn la_cloture_d_exercice_ne_parle_pas_comme_un_panneau() {
        const ATTENDUS: [(&str, &str); 3] = [
            ("de-CH", "Abschliessen"),
            ("it-CH", "Chiudi l’esercizio"),
            ("en-CH", "Close fiscal year"),
        ];
        for (locale, attendu) in ATTENDUS {
            let v = valeurs_brutes(locale);
            let panneaux: Vec<(&String, &String)> = v
                .iter()
                .filter(|(k, _)| k.ends_with("-close") || k.ends_with("-dismiss"))
                .collect();
            assert!(!panneaux.is_empty(), "{locale} : aucune clé de panneau");
            for cle in [
                "fiscal-year-close-button",
                "fiscal-year-close-confirmation-action",
            ] {
                let valeur = &v[cle];
                assert_eq!(valeur, attendu, "{locale} : {cle}");
                for (k, p) in &panneaux {
                    assert_ne!(valeur, *p, "{locale} : {cle} parle comme {k}");
                }
            }
        }
        // de-CH : « geschlossen / schliessen / Schliessung » nu est réservé aux panneaux ;
        // ailleurs, seule la forme séparable « Schliessen Sie … ab » (= abschliessen).
        let de = valeurs_brutes("de-CH");
        let nu = regex::Regex::new(r"\b([Gg]eschlossen|[Ss]chliessen|[Ss]chliessung)\b").unwrap();
        let separable = regex::Regex::new(r"[Ss]chliessen Sie [^.;:]* ab\b").unwrap();
        let (mut panneaux, mut separables) = (0, 0);
        for (cle, valeur) in &de {
            if !nu.is_match(valeur) {
                continue;
            }
            if cle.ends_with("-close") || cle.ends_with("-dismiss") {
                panneaux += 1;
            } else {
                assert!(
                    separable.is_match(valeur),
                    "de-CH : {cle} emploie le verbe du panneau pour un exercice : {valeur}"
                );
                separables += 1;
            }
        }
        assert!(panneaux > 0 && separables > 0, "{panneaux} / {separables}");
    }
}
