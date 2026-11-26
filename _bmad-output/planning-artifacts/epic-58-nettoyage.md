# Epic 58 — Nettoyage : petites dettes P3/P4 regroupées par module

*Kickoff préparé le 2026-11-26 (décision de Guy du 2026-11-26 : un epic de nettoyage par release ; le premier est l'Epic 58).* Branche `epic/58-nettoyage`, à partir de `main` au lancement de sa première story. Release visée : **v0.21.0, vendredi 2026-12-04** (la release qui suit la v0.20.0). L'epic n'entre dans `main` que terminé. **Projet de fiche : rien n'est créé ni poussé ; les affectations à l'epic (champ *Epic* du projet GitHub, milestone « Vendredi 2026-12-04 ») sont à faire par le Project Lead.**

## Objet

Réduire le stock d'issues ouvertes sans rien ajouter à Kesh : soixante issues P3 ou P4 peu coûteuses, regroupées en six stories d'un même module, les plus anciennes d'abord, celles qui touchent l'usage réel du Project Lead (numéro de facture au format par défaut, rapprochement, factures clients) avant les autres.

## Règle

Au plus six stories, **chacune regroupant cinq à dix issues P3 ou P4 peu coûteuses d'un même module** (textes et manuel, tests, cosmétique d'un domaine, dette d'un crate), **sans règle métier changée, ni migration, ni changement d'API**. Une issue qui révèle en cours de route un vrai défaut sort du lot et suit son propre chemin (issue, priorité P1/P2 si elle fausse les livres). Chaque story garde sa boucle de revue et ses gates ; pas de CHANGELOG écrit dans les stories (notes de version des fiches) ; aucun PDF commité. Cinq stories sur six ne touchent aucun code de production ; la 58-4 et la 58-5 touchent des textes et du CSS visibles : recette visuelle exigée.

## Sélection

Source : `gh issue list --state open --limit 900 --label P3|P4` du 2026-11-26 (601 issues P3/P4 ouvertes). Écartées : les 112 `enhancement` ; celles qui portent un élément de projet ; celles dont le titre ou le corps nomme les Epics 37 à 40, 41, 45, 46, 47, 49 à 57 (53 issues, détectées par motif `Epic NN` : **un corps qui cite un epic seulement comme origine a pu être écarté à tort**, d'où la réserve de la 58-5 sur #1549, #1552, #1553) ; celles dont le corps laisse voir une règle métier, une migration ou une API (ex. #1114, #1116, #1121, #709, #726). Ordre de prise : par module, **les plus anciennes d'abord** ; l'usage réel prime à ancienneté égale (58-1). Vérification « règle neuve » : chaque issue retenue a été relue au code de `origin/main` (7b717e8cf) quand le corps nomme un fichier ou une ligne : voir `fermetures-proposees.md` § 2 pour ce qui a été écarté parce que déjà corrigée. Aucune issue retenue ne s'est révélée entièrement corrigée ; #1043 l'est en partie et reste dehors.

## Stories

| Story | Module | Issues | Plus ancienne |
|---|---|---|---|
| 58-1 | Format de numéro de facture : textes, tests, miroir | 8 | #1115 |
| 58-2 | Comptes désignés et tests Rust de revue | 10 | #592 |
| 58-3 | Manuel et guide de l'API | 10 | #730 |
| 58-4 | Textes d'interface et catalogues i18n | 10 | #324 |
| 58-5 | Cosmétique des écrans | 10 | #253 |
| 58-6 | Tests frontend de la réconciliation | 10 | #1078 |

Total : 8 + 10 × 5 = **58 issues** (recomptées depuis les tableaux ci-dessous). Six stories : la borne tient, sans story de solde séparée ; si le Project Lead veut une story de solde (règle de fin d'epic), elle prend la place de la 58-6, la moins ancienne. Ordre conseillé : 58-3 (sans code) d'abord, 58-2 et 58-6 (tests) ensuite, 58-1, 58-4 et 58-5 (écrans) à la fin, pour laisser le temps de la recette visuelle. Ordre des plus anciennes : 58-5/#253, 58-4/#324, 58-3/#730, 58-2/#592, 58-1/#1115, 58-6/#1078.

### Story 58-1 — Format de numéro de facture : textes, tests et miroir

**Module** : `kesh-core` (`invoice_format.rs`), `kesh-api` (tests), `frontend/src/lib/features/invoices/` — usage réel du Project Lead (numéro de facture au format par défaut).

| # | Prio | Créée | Titre | Correctif attendu | Coût |
|---|---|---|---|---|---|
| #1115 | P4 | 2026-11-13 | Refus NUMBER_FORMAT_UNRENDERABLE : les phrases too_long et duplicate présument leur c | Une phrase par sous-cas (ou formulation neutre) pour `too_long` et `duplicate` | S |
| #1117 | P4 | 2026-11-13 | Test manquant : format par défaut F-{YEAR}-{SEQ:04} sur deux exercices qui commencent | Test d'API : F-{YEAR}-{SEQ:04} sur deux exercices commencés la même année (cause `duplicate`) | S |
| #1118 | P4 | 2026-11-13 | invoice_format : MAX_RENDERED_LEN et le Display de RenderedTooLong parlent encore de  | Commentaire de `MAX_RENDERED_LEN` et `Display` de `RenderedTooLong` : dire « octets » | S |
| #1119 | P4 | 2026-11-13 | Miroir du format de numéro : {SEQ:+4} refusé à l'écran, accepté par le serveur | Aligner le commentaire du miroir sur `u32::from_str` et trancher `+` (texte seulement si le serveur reste souple) | S |
| #1120 | P4 | 2026-11-13 | Miroir du format de numéro : \s admet tout blanc Unicode et length compte en UTF-16,  | Miroir : `\s` → espace/tabulation, longueur en octets (comme `parse`) | S |
| #1122 | P4 | 2026-11-13 | Choix C-e351-7 : la borne de 64 octets présentée comme imposée par la QR-facture, alo | Registre et textes : la borne de 64 octets est un choix de prudence, non une contrainte QR | S |
| #1123 | P4 | 2026-11-13 | Message too_long : « une lettre accentuée en compte deux » sous-dit le poids des cara | Message `too_long`, manuel, `api-external.md`, CHANGELOG : « deux ou trois octets » | S |
| #1124 | P4 | 2026-11-13 | Tests 35-1 : le compteur n'est pas asserté pour le nom multi-octets ni pour le format | Asserter que le compteur est intact (nom multi-octets, format illisible) | S |

**8 issues.** **Gates** : gate Rust ciblé (`binary(number_format_unrenderable_e2e)`, `kesh-core`), vitest `invoice-number-format`, gate complet + E2E complet à la clôture ; manuels des 4 langues si le texte du refus change ; PDF non commités.

### Story 58-2 — Comptes désignés et tests Rust de revue

**Module** : `kesh-api/tests/` (`designated_account_type_e2e.rs`, `i18n_e2e.rs`, `accounts_e2e.rs`), `kesh-db/tests/supplier_invoices_repository.rs`, un test frontend d'i18n.

| # | Prio | Créée | Titre | Correctif attendu | Coût |
|---|---|---|---|---|---|
| #592 | P3 | 2026-10-09 | Les motifs d'écran de modification bloquée : l'identité de leur clé i18n n'est gardée | Test Vitest qui tient la clé i18n des douze motifs `journal-entries-modify-blocked-*` | M |
| #732 | P3 | 2026-10-24 | Tests de la 32-1a : quatre trous de couverture relevés en revue (LOW) | Quatre trous de couverture de la 32-1a (mutations survivantes) | M |
| #736 | P3 | 2026-10-24 | i18n_e2e : deux tests réinitialisent la locale globale sans le verrou i18n (flake pos | `get_i18n_lock()` dans les deux tests de `i18n_e2e.rs` (l. 128, 153) | S |
| #1060 | P3 | 2026-11-13 | Réglages de facturation : seul « attendu : actif » est asserté ; « attendu : passif » | Asserter « passif » et « charge ou produit » (`account_type_in_words`) | S |
| #1062 | P3 | 2026-11-13 | Compte désigné de mauvais type : aucun test d'un même compte faux pour deux réglages | Test : un même compte faux pour deux réglages figure deux fois | S |
| #1063 | P3 | 2026-11-13 | Validation fournisseur refusée pour un compte de mauvais type : le test HTTP ne relit | Relire la facture fournisseur après le refus : brouillon, numéro interne nul | S |
| #1064 | P3 | 2026-11-13 | Avoir sur une facture dont la créance a été retypée depuis : aucun test ne fixe le co | Test : un avoir reste possible quand la créance a été retypée | S |
| #1066 | P4 | 2026-11-13 | Test de la Story 35-3, AC4 : le rôle Payable n'est vérifié qu'après le retypage | Asserter le rôle Payable avant le retypage | S |
| #1127 | P4 | 2026-11-13 | Tests de la 35-5a : rouges et mutations relevés sur un seul site des tests qui en cou | Rouges et mutations relevés sur chaque site, non sur un seul | M |
| #1433 | P3 | 2026-11-20 | Tests manquants du PUT /accounts/{id} avec parentId : version périmée et parentId abs | Tests `PUT /accounts/{id}` : `parentId` avec version périmée (409) ; `parentId` absent | S |

**10 issues.** **Gates** : gate ciblé par binaire touché ; `kesh-db` : gate complet dès qu'un repository bouge (aucun attendu) ; aucune production touchée.

### Story 58-3 — Manuel et guide de l'API : écarts avec le code

**Module** : `docs/manual/{fr,de,it,en}/` (.tex seulement), `docs/api-external.md`, `CHANGELOG.md`, `.env.example`, `docs/migrations-idempotence-audit.md`, `post_restore.rs` (texte), `already-booked.ts` (commentaire).

| # | Prio | Créée | Titre | Correctif attendu | Coût |
|---|---|---|---|---|---|
| #730 | P3 | 2026-10-24 | Manuel d'administration : le rôle Consultation est dit « sans mutation », alors que c | Rôle Consultation : ne plus dire « aucune mutation » (écriture `report.generated`) | S |
| #748 | P3 | 2026-10-25 | Audit d'idempotence et justifications d'exemption : « inscrite au registre » et « dan | Audit d'idempotence et exemptions : retirer « inscrite au registre » / « dans la fenêtre » périmés | S |
| #767 | P3 | 2026-10-25 | Manuel : le fichier de log s'appelle kesh.<date>.log, non kesh.log | `kesh.<date>.log` au manuel (l. 1031, 1037, 1045) et à `.env.example` (l. 319, 331, 334, 350) | S |
| #814 | P3 | 2026-10-29 | Manuel d'administration, serveur MCP : quatre imprécisions relevées en revue de la v0 | Quatre imprécisions du manuel MCP (« en attente »…) | S |
| #1061 | P4 | 2026-11-13 | Guide de l'API : « triés par numéro » pour ACCOUNT_NOT_POSTABLE et DESIGNATED_ACCOUNT | « par numéro, en ordre de chaîne » (`api-external.md`, CHANGELOG) | S |
| #1067 | P4 | 2026-11-13 | Manuel utilisateur, compte de produit par défaut : son type n'est contrôlé que si une | Manuel : « restent contrôlés à la validation dès qu'une ligne le suit » (l. 537) | S |
| #1074 | P3 | 2026-11-13 | Manuel et CHANGELOG : « une espace ou une apostrophe après le séparateur décimal » es | « entre le séparateur décimal et le dernier chiffre » (manuel l. 2969, CHANGELOG l. 132) | S |
| #1076 | P4 | 2026-11-13 | CHANGELOG et manuel : « 1'234,567 » donné pour exemple de « milliers déjà séparés »,  | Exemple « 1'234,567 » : retirer l'explication « milliers déjà séparés » | S |
| #1087 | P4 | 2026-11-13 | Manuel, réconciliation : après la phrase ajoutée sur Rejeter, le « Elle » qui suit a  | « Elle reste » → « La transaction reste » (manuel l. 3744) | S |
| #1405 | P3 | 2026-11-19 | Manuel utilisateur : la fiche contact décrit un champ « Compte comptable par défaut » | Retirer « Compte comptable par défaut » de la fiche contact (manuel l. 1385) | S |

**10 issues.** **Gates** : aucun code exécutable (hors commentaire) : pas de gate Rust/E2E ; vérification du manuel contre le code, PDF aplatis ; `textes_coherents.rs` et `npm run lint-i18n-ownership` si un texte du catalogue change ; PDF régénérés à la release seulement.

### Story 58-4 — Textes d'interface et catalogues i18n

**Module** : `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`, `frontend/src/routes/(app)/+layout.svelte`, `JournalEntryForm.svelte`, liste des factures.

| # | Prio | Créée | Titre | Correctif attendu | Coût |
|---|---|---|---|---|---|
| #324 | P3 | 2026-08-20 | [KF-042] Le faux ami « Valider » — journal-entry-form-submit dit Speichern/Salva/Save | Clé du bouton d'écriture : un libellé distinct de « Enregistrer » dans les 4 langues | S |
| #339 | P4 | 2026-08-23 | [KF-046] Libellés français en dur dans le MARKUP — l'angle mort qui reste après l'Epi | Libellés français en dur dans le balisage (liste des factures, puis relevé du même motif) | M |
| #844 | P4 | 2026-10-30 | [DETTE] Clé i18n morte : homepage-bank-no-transactions | Supprimer la clé morte `homepage-bank-no-transactions` (4 locales) | S |
| #861 | P4 | 2026-10-31 | [bug] Coque : le placeholder de recherche et le pied de page restent en français dans | Placeholder « Rechercher... » et pied de page via `i18nMsg` | S |
| #873 | P4 | 2026-11-03 | Refus LATER_FISCAL_YEAR_CLOSED : l'exercice nommé contredit la consigne « en commença | `LATER_FISCAL_YEAR_CLOSED` : nom de l'exercice cohérent avec la consigne | S |
| #874 | P4 | 2026-11-03 | it-CH : forme d'adresse mêlée (« tu » et « voi ») dans les refus | it-CH : une seule forme d'adresse dans les refus | S |
| #908 | P4 | 2026-11-05 | PERIOD_LOCKED : les messages du serveur affichent des dates ISO | `PERIOD_LOCKED` : dates au format suisse dans les messages | S |
| #1049 | P4 | 2026-11-12 | fr-CH : apostrophe droite dans le message RETAINED_EARNINGS_LINE (« d'ouverture ») | fr-CH : apostrophe typographique dans `RETAINED_EARNINGS_LINE` | S |
| #1086 | P4 | 2026-11-13 | Catalogue fr-CH : aucune espace insécable autour des guillemets « », qui se retrouven | fr-CH : espaces insécables autour des guillemets « » | S |
| #1491 | P4 | 2026-11-23 | Audit : le libellé « Doublon de facture fournisseur confirmé » dit le contraire du ge | Libellé d'audit « Doublon de facture fournisseur confirmé » : dire le geste | S |

**10 issues.** **Gates** : gate frontend (`check`, `lint-i18n-ownership`, `test:unit`, `build`) ; compteur `sitesTotal` de `i18n-keys.test.ts` à recompter ; gate Rust ciblé sur `kesh-i18n` ; E2E complet à la clôture (libellés) ; recette visuelle : captures de l'écran d'écriture, de la liste des factures et de l'en-tête.

### Story 58-5 — Cosmétique des écrans : colonnes collées, débordements, contraste

**Module** : `frontend/src/routes/(app)/audit-log`, `lib/features/bank-import`, `reconciliation`, `accueil`, `app.css`.

| # | Prio | Créée | Titre | Correctif attendu | Coût |
|---|---|---|---|---|---|
| #253 | P4 | 2026-07-15 | [KF] Contraste AA insuffisant sur les chips sélectionnés (bg-primary-light + texte fo | Contraste AA des chips sélectionnés (3,97:1 → 4,5:1) | S |
| #933 | P4 | 2026-11-06 | [bug] Aperçu de l'import bancaire : les colonnes Montant et Détails se touchent | Marge entre « Montant » et « Détails » (aperçu d'import) | S |
| #996 | P3 | 2026-11-10 | Contraste : --color-warning (#d97706) sur blanc mesure 3,19:1, sous le seuil AA pour  | `--color-warning` à 3,19:1 : assombrir le jeton | S |
| #1008 | P4 | 2026-11-11 | Accueil, tuile Comptes bancaires : le lien « Solde non disponible » chevauche le nom  | Tuile « Comptes bancaires » : le lien « Solde non disponible » chevauche le nom | S |
| #1015 | P4 | 2026-11-11 | Société, avis de conflit : une valeur longue sans espace déborde à largeur de télépho | Avis de conflit société : valeur longue sans espace qui déborde (`overflow-wrap`) | S |
| #1016 | P4 | 2026-11-11 | Réconciliation : la colonne Montant colle à la colonne Contrepartie, en-tête et cellu | Réconciliation : marge entre « Montant » et « Contrepartie » | S |
| #1096 | P4 | 2026-11-13 | Fiche facture validée : à 1400 px, la barre d'actions déborde et coupe le dernier bou | Fiche facture à 1400 px : la barre d'actions coupe le dernier bouton (retour à la ligne) | S |
| #1153 | P4 | 2026-11-14 | Réconciliation : les titres de colonnes « MontantContrepartie » et « Candidate » sont | Libellés de colonnes « MontantContrepartie » et « Candidate » | S |
| #1154 | P4 | 2026-11-14 | Journal d'audit : la colonne « N° » est collée à la colonne « Détails » | Journal d'audit : marge entre « N° » et « Détails » | S |
| #1184 | P4 | 2026-11-14 | Champ de compte : le bouton × chevauche la fin d'un libellé long | Champ de compte : le bouton × chevauche un libellé long | S |

**10 issues.** **Gates** : gate frontend ; **recette visuelle obligatoire** (captures avant/après de chaque écran touché, thème clair, base de démonstration) ; E2E complet à la clôture ; contraste : contrôle axe des specs existantes.

### Story 58-6 — Tests frontend de la réconciliation (35-4, 35-7)

**Module** : `frontend/src/lib/features/reconciliation/*.test.ts`, `journal-entries/AccountAutocomplete*`, `frontend/tests/e2e/reconciliation-*.spec.ts`.

| # | Prio | Créée | Titre | Correctif attendu | Coût |
|---|---|---|---|---|---|
| #1078 | P3 | 2026-11-13 | E2E 35-7 (AC7) : l'assertion sur le corps du POST passe aussi sur le code d'avant ; s | E2E 35-7 : l'assertion sur le corps du POST doit rougir sur l'ancien code | S |
| #1079 | P3 | 2026-11-13 | Vitest 35-7 (AC3, Écriture manuelle) : l'assertion « champ vide » passe sans l'effet  | Vitest : « champ vide » doit dépendre du `$effect` de `keepOfferedCounterparty` | S |
| #1080 | P4 | 2026-11-13 | Vitest 35-7 : les tests « comptes restés proposés » attendent un état déjà vrai avant | Vitest : « comptes restés proposés » attendent un état déjà vrai avant la résolution | S |
| #1082 | P4 | 2026-11-13 | E2E 35-7 (AC7) : l'option est choisie par dispatchEvent('mousedown'), non par un vrai | E2E : choisir l'option par un vrai clic, non `dispatchEvent('mousedown')` | S |
| #1083 | P4 | 2026-11-13 | Vitest : Éclater à trois lignes dont deux sur le compte de la banque n'est pas testé | Vitest : Éclater à trois lignes dont deux sur le compte de la banque | S |
| #1084 | P4 | 2026-11-13 | Champ de compte strict : un numéro complet tapé puis Tab est jeté, sans test qui fixe | Test qui fixe le comportement de « numéro complet puis Tab » | S |
| #1088 | P4 | 2026-11-13 | Réconciliation : la garde « if (!current) return; » de confirmGesture n'est plus exer | Retirer ou exercer la garde `if (!current) return;` de `confirmGesture` (l. 520) | S |
| #1090 | P3 | 2026-11-13 | E2E 35-4 : cliquerHors réessaie le clic jusqu'à la fermeture, ce qui masquerait un pr | E2E 35-4 : `cliquerHors` ne doit pas réessayer le clic jusqu'à la fermeture | S |
| #1091 | P4 | 2026-11-13 | E2E 35-4 : toContainText('1') accepte « 10 » ou « 11 » comme nombre de transactions c | E2E 35-4 : `toContainText('1')` accepte « 10 » ; asserter le nombre exact | S |
| #1093 | P4 | 2026-11-13 | Vitest 35-4 : bornes du nombre de cochées non testées (trois cochées dont une signalé | Vitest : bornes du nombre de cochées (trois dont une signalée ; toutes signalées) | S |

**10 issues.** **Gates** : vitest ciblé par fichier, puis E2E ciblés (`reconciliation-already-booked-reject`, `reconciliation-split`, `reconciliation-manual-entry`), E2E complet à la clôture ; aucun code de production attendu (hors garde #1088).
## Fichiers et modules touchés (déclarés au kickoff)

- `docs/manual/{fr,de,it,en}/*.tex` (jamais les PDF), `docs/api-external.md`, `CHANGELOG.md` (une ligne au plus, à l'intégration), `.env.example`, `docs/migrations-idempotence-audit.md` : 58-3, 58-1.
- `crates/kesh-i18n/locales/*/messages.ftl` et `frontend/src/lib/shared/i18n-keys.test.ts` (`sitesTotal`) : 58-4, 58-1 (si le texte du refus change).
- `crates/kesh-core/src/invoice_format.rs`, `crates/kesh-api/tests/number_format_unrenderable_e2e.rs`, `frontend/src/lib/features/invoices/invoice-number-format.ts` : 58-1.
- `crates/kesh-api/tests/*`, `crates/kesh-db/tests/supplier_invoices_repository.rs` : 58-2.
- `frontend/src/routes/(app)/+layout.svelte`, `audit-log/+page.svelte`, `frontend/src/app.css`, composants de `bank-import`, `reconciliation`, `invoices`, `journal-entries` : 58-4, 58-5, 58-6.

## Recoupements avec les autres epics

Contrôle à refaire au lancement par `git diff --name-only main...<branche>` sur les branches ouvertes (non fait ici : lecture seule, aucune branche créée).

- **Epics d'un contributeur (37 à 40, 46, 47, 55, 56)** : les catalogues i18n et `i18n-keys.test.ts` sont communs ; règle du CLAUDE.md (chacun son bloc, conflit résolu en gardant les deux côtés, `sitesTotal` recompté).
- **Epic 53 (écritures justes)** : `journal_entries.rs` et les tests de comptes désignés (58-2) côtoient la 53-2 (TVA) ; aucun fichier de production commun attendu.
- **Epics 41, 45, 49 à 52, 54, 57** : les issues qu'ils portent sont exclues ; les pages `audit-log`, `reconciliation` et la fiche facture (58-5) peuvent recouper les écrans de l'Epic 45 (note interne) et de l'Epic 51 : à contrôler.
- **Epic 54 (décompte de TVA)** : manuel utilisateur (58-3), même `.tex` : conflits de texte possibles, second arrivé résout.

## Dépendances

Dépend de : aucune. Bloque : aucune. Les stories sont indépendantes entre elles ; 58-1 et 58-4 partagent les catalogues i18n (la seconde fusionnée reprend la première).

## Décomptes au lancement (recomptés le 2026-11-26)

| | Valeur |
|---|---|
| Issues ouvertes | **690** (`gh issue list --state open --limit 900`) ; le décompte à `--limit 600` du CLAUDE.md est tronqué à 600 et ne donne plus le total |
| Bugs (`bug` ou `known-failure`) | **302** |
| P1 ouvertes | **8** |
| Issues P3 + P4 ouvertes | 601 (297 + 304) |
| Issues de l'epic | **58** |
| Fermetures proposées hors epic (doublons) | 3 (#1548, #1550, #1551) + 1 sous réserve (#125) |

**Objectif au tag (non barrière)** : moins de **690 − 58 − 3 = 629** issues ouvertes si les 58 issues sont réglées et les doublons fermés ; bugs : moins de 302 ; le nombre d'issues de l'epic qui portent `bug` n'a pas été compté ici, à recompter au lancement pour fixer l'objectif. Le chiffre du tag précédent (v0.20.0) reste à écrire par l'orchestrateur à la PR de release.

## Hésitations

- Le motif d'exclusion « Epic NN » est large : à revoir si le Project Lead veut récupérer les issues nées de l'Epic 49 (#1549, #1552, #1553, #1423, #1399…).
- 58-1 touche des textes de refus visibles de l'API (messages d'erreur) : sans changer de code d'erreur, mais à confirmer comme « pas de changement d'API » (aucun champ ni code, seulement la phrase).
- Les coûts S/M sont estimés au corps des issues, non mesurés ; 58-2 contient trois M (#592, #732, #1127).
