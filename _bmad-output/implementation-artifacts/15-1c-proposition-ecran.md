# Story 15.1c : L'écran « Postes ouverts » — index (découpée)

## Status

split

⛔ **CORPS VIDÉ — cette fiche ne contient plus ni décisions, ni critères, ni tâches** *(découpée le 2026-10-09 à la
remédiation de sa validation P1, registre C-15-1c-1 ; partie serveur extraite en 15-1c-0 à la remédiation de sa
validation P2, C-15-1c-14)*. Elle garde les pointeurs vers ses trois sous-fiches, la table qui dit où chaque critère
et chaque point reçu est allé, le bilan des validations P1 et P2 et l'historique (Change Log).
Le corps d'avant le découpage se lit dans l'historique git
(`git show 8e36a146:_bmad-output/implementation-artifacts/15-1c-proposition-ecran.md`). *(Précédents : 15-1, 15-12,
15-1a — C124, 15-1a2 — C-15-1a2-1.)* **La numérotation des critères est conservée** dans les sous-fiches ; les
numéros neufs commencent à AC15. Le nom de fichier est gardé (C99 : les clés du registre et du `sprint-status` y
renvoient).

## Les trois sous-fiches

| ordre | fiche | ce qu'elle porte |
|---|---|---|
| 0 | `15-1c-0-groupe-de-lettrage-enrichi.md` | **Le serveur** : `GET /letterings/{key}` enrichi (journal, libellé, pièce, possession, période par ligne ; compte ; `manualDissolutionBlockedBy`), par réemploi de la part « pièce et période » de la requête B de la 15-1b ; les refus 1 à 3 de la dissolution passés par **une** fonction d'ordre à étapes (évaluation paresseuse, détails inchangés) ; `api-external.md`, entrée *Modifié* du CHANGELOG, README. `refs #518` |
| 1 | `15-1c-i-ecran-postes-ouverts.md` | **L'écran `/open-items`** : compte (lettrables, archivés compris) et date, la liste et ses motifs (`reason`, `documentState`, `amountDue`), le pied avec son sens (débiteur, créditeur), la sélection et le lettrage (pages, plafond de 200, `big.js`, tout 404/409 recharge), les propositions (chargées à part, échec indépendant, `reversalPair`, proposition périmée), **le groupe et le délettrage** (état d'URL `?group=`, compte devenu non lettrable), le bandeau de frontière, les rôles, le menu, l'i18n, le composant de lien de code, l'E2E lettrer → délettrer, la ligne du README. `refs #518` |
| 2 | `15-1c-ii-lettrage-dans-kesh.md` | **Le lettrage dans le reste de Kesh** : la colonne « Lettrage » de la fiche d'écriture et du Grand livre (`colspan` calculés), le lien « Postes ouverts de ce compte », le lien du motif `ENTRY_LETTERED`, les exports exclus, le **manuel** (section neuve, propagation par inventaire, PDF), le **CHANGELOG** (une entrée *Ajouté*), `api-external.md`, le README, le site, la brochure. **`closes #518`** |

**Dépendances** : la 15-1c-0 suppose la 15-1b mergée (et donc la 15-1b-0 et les 15-1a2-*) ; la 15-1c-i suppose la
15-1c-0 ; la 15-1c-ii suppose la 15-1c-i. Ordre complet : **15-12a → 15-12b → 15-1a-i → 15-1a-ii → 15-1a2-0 →
15-1a2-i → 15-1a2-ii → 15-1b-0 → 15-1b → 15-1c-0 → 15-1c-i → 15-1c-ii**. ⛔ **Pas de tag v0.13.0 entre la 15-1c-i et la 15-1c-ii** (C124, étendue par C-15-1c-1) :
entre les deux, le manuel et le CHANGELOG disent encore que le délettrage se fait par l'API. Les **E2E** de la 15-1c-i
et de la 15-1c-ii ne sont atteignables qu'une fois la 15-1c-0 mergée (vue, propositions, `letterable`) ; le scénario (1) de
la 15-1c-i suppose la 15-1a2-i (groupes `document`).

## Pourquoi le découpage

Findings **R-10 = F-10** de la validation P1 : la fiche se déclarait « cinq modules, au seuil » ; au recompte, huit
à onze au grain fin, et — après les findings R-2 = F-2 — un enrichissement serveur de plus. Décision de
l'orchestrateur, **C-15-1c-1** : couture **l'écran** (tout ce qui vit à `/open-items`, y compris le groupe et le
délettrage, et le serveur qu'il exige) / **le lettrage dans le reste de Kesh** (les autres écrans qui le montrent,
et toute la documentation). Comptes aux deux grains (C-15-1a2-21) : 15-1c-i — 4 crates/paquets, 9 au grain fin
(cinq de logique, quatre mécaniques et un paragraphe de documentation) ; 15-1c-ii — 2 et 11 (trois de logique plus
l'E2E, sept supports de texte). Dérogation écrite dans chacune (**C-15-1c-11**, patron C-15-1a2-23).

**Puis la coupe « serveur d'abord »** (validation P2, F2-3 ; décision de l'orchestrateur, **C-15-1c-14**) : les onze
MEDIUM de la P2 étaient tous nés de la remédiation P1, et le motif qui écartait cette coupe — « une route enrichie
sans écran qui la lise n'est pas livrable seule » — est réfuté par la 15-1b, story serveur livrée sans écran. La
**15-1c-0** prend AC15, AC16 et leurs tests ; la refonte de la dissolution — primitive unique du retrait de la
marque — se relit seule, et la 15-1c-i ne touche plus aucun repository. Comptes : 15-1c-0 — 2 crates, 5 au grain
fin (deux de logique, trois supports de texte), sans dérogation ; 15-1c-i — 2 crates/paquets, 8 au grain fin (trois
de logique) ; 15-1c-ii inchangée.

## Table de correspondance — où chaque critère est allé

| critère de la 15-1c (corps au `8e36a146`) | 15-1c-i | 15-1c-ii |
|---|---|---|
| *(la 15-1c-0 porte AC15, AC16, AC18 — voir les lignes neuves)* | | |
| AC1 — compte et date | ✓ (réécrit : `letterable`, archivés, `asOf` explicite, 409/404) | — |
| AC2 — la liste | ✓ (exercice avec le numéro, table des liens de pièce) | — |
| AC3 — les motifs | ✓ (réécrit en deux dimensions, `nothingDue`) | — |
| AC4 — lettrer à la main | ✓ (`manuallyLetterable`, pages, 200, `big.js`, `inOpenPeriod`, refus et rechargements) | — |
| AC5 — les propositions | ✓ (chargement à part, `asOf`, `total`, périmée) | — |
| AC6 — voir et défaire un groupe | ✓ (URL, groupe lu enrichi — 15-1c-0 AC15 —, code `LETTERING_ALL_LINES_IN_CLOSED_PERIODS`) | — |
| AC7 — bandeau de frontière | ✓ | — |
| AC8 — écart avec la Balance | ✓ (**phrase réfutée remplacée** : le total égale le solde que montre la Balance) | (stabilité « au X » : au manuel, AC12) |
| AC9 — le code visible ailleurs | — | ✓ (fiche, Grand livre, `ENTRY_LETTERED`) |
| AC10 — rôles | ✓ | (motif `ENTRY_LETTERED` : rôle inchangé) |
| AC11 — i18n | part i | part ii |
| AC12 — manuel | — | ✓ (règle C105, inventaire) |
| AC13 — E2E | part i, scénarios (1)–(6) | part ii, scénarios (7)–(9) |
| AC14 — CHANGELOG | — | ✓ |
| — | **AC15** (neuf) → **15-1c-0** : `GET /letterings/{key}` enrichi | — |
| — | **AC16** (neuf) → **15-1c-0** : sa documentation dans `api-external.md` | — |
| — | — | **AC17** (neuf) : `api-external.md` (« l'écran viendra »), README, site, brochure |
| — | **AC18** (neuf) → **15-1c-0** : CHANGELOG *Modifié*, README | — |
| — | **AC19** (neuf) : README | — |

## Intégration des sections reçues

**« Reçu de la 15-1a »** (points 1 à 11 du corps au `8e36a146`) : 1 (code `LETTERING_ALL_LINES_IN_CLOSED_PERIODS`) →
15-1c-i AC4, AC6 ; 2 (règle des périodes au manuel) → 15-1c-ii AC12 ; 3, 9 (glossaire, phrase provisoire) → 15-1c-ii
AC12 point 2 ; 4 (`LETTERING_LINE_OWNED_BY_DOCUMENT` neutre) → 15-1c-i AC6, 15-1c-0 AC15 ; 5, 6 (ordre) → Status des deux ;
7 (sixième condition de *Modifier*, « par l'API dans cette version ») → 15-1c-ii AC12 ; 8 (`ENTRY_LETTERED` en
dernier) → 15-1c-ii AC9 ; 10 (CHANGELOG) → 15-1c-ii AC14 ; 11 (`fiscalYearName`) → 15-1c-i AC2, AC6, 15-1c-0 AC15.

**« Pour la 15-1c »** de la 15-1b (points 1 à 13) : 1 → 15-1c-i AC8 ; 2 → AC3 ; 3 → AC4 ; 4, 11, 13 → AC5 ; 5 → AC6
(et l'enrichissement de la 15-1c-0 AC15 pour les lignes d'un groupe) ; 6 → AC1 (`letterable`) et 15-1c-ii AC9 (`letteringCode` du
Grand livre) ; 7 → AC1, AC6 ; 8 → 15-1c-ii AC12 ; 9 → 15-1c-ii AC12 ; 10 → AC5 et 15-1c-ii AC12 ; 12 → AC3, AC4.

## Change Log

### Validation P4 ciblée — 2026-10-09 (Haiku 4.5, une lentille, commit `99280a24`) — VALIDATION CLOSE

Prompt versionné : `15-1c-validate-prompt-p4-ciblee.md`. Rapport : `/home/gcorbaz/devel/kesh-gate-logs/15-1c-validate-p4-F.md`.
**0 CRITICAL / 0 HIGH / 0 MEDIUM / 4 LOW**, hunks examinés et non examinés déclarés. Vérifiés par l'orchestrateur :
F-1 (15-1c-0 : `journal_entries.rs:2578` appelle `is_letterable_account`) — **corrigé** ; F-2 (15-1c-0, test 9 :
accents graves imbriqués) — **corrigé** (`grep -nF`) ; F-3 (15-1c-ii : « 11 au grain fin » non reproductible) —
**réfuté** (3 de logique + E2E + 7 supports de texte = 11, la brochure étant dans `docs/manual/fr`) ; F-4 (15-1c-i :
« exception assumée » à contresens de la règle qu'elle applique) — **corrigé** (« pas d'exception », AC4, test 7,
C-15-1c-24). La lentille a créé puis supprimé un fichier de travail dans `kesh-gate-logs/` (déclaré par elle ;
`ls` : absent).

**Remédiation documentaire seulement** — aucune règle, aucun contrat, aucun comportement prescrit ne change : la
boucle se clôt (`CLAUDE.md` § « La passe ciblée », critère de clôture). **Trend** : P1 (Sonnet ×2) 4 HIGH / 16 MEDIUM
bruts → découpage ; P2 (Opus ×2) 0 HIGH / 7 MEDIUM distincts, tous nés de la P1 → extraction de la 15-1c-0 ; P3
(Sonnet ×2, trois fiches) 0 MEDIUM / 19 LOW bruts ; P4 ciblée (Haiku) 0 MEDIUM / 4 LOW (3 justes, 1 réfuté).
**VALIDATION CLOSE** sur les trois fiches : 15-1c-0, 15-1c-i, 15-1c-ii, ready-for-dev dans l'ordre … → 15-1b →
15-1c-0 → 15-1c-i → 15-1c-ii.

### Validation P3 — 2026-10-09 (Sonnet 5.5 ×2, lentilles R et F ; remédiation Opus 5.5, en autonomie) — 0 AU-DESSUS DE LOW

Prompt versionné : `15-1c-validate-prompt-p3.md` (remédiation P2 visée : `82524343`). Rapports :
`/home/gcorbaz/devel/kesh-gate-logs/15-1c-validate-p3-R.md` et `-F.md`. **R : 0 CRITICAL / 0 HIGH / 0 MEDIUM / 10 LOW ;
F : 0 / 0 / 0 / 9 LOW**, sur les **trois** fiches (première passe de la 15-1c-0). Les deux lentilles déclarent leurs
axes exercés et non exercés ; l'orchestrateur a recoupé les « 0 » au code de `f9b6b199` (séquence
`dissolve_group_in_tx` : verrous → refus 1 → propriété si `Reversal` → borne → refus 3 ; statuts 400/404/409 de
`kesh-api/src/errors.rs` ; `colspan` du Grand livre et pied de la fiche d'écriture ; sites « lettr » des trois `.tex` :
24 + 5 + 1 ; `LineRow` partagé ; `letterable_account` sans `name`). Recoupements : R L-2 = F-5 ; R L-5 = F-1 ;
R L-6 ≈ F-9 (part) ; R L-9 ≈ F-9 (part). **Tous les LOW appliqués** :

| finding | fiche | sort |
|---|---|---|
| R L-1 | 15-1c-0 | deux pièces partagées nommées (`kesh-db` : propriétaires et périodes ; `kesh-api` : constructeur de `document`) ; unicité du code prouvée par `grep` |
| R L-2 = F-5 | 15-1c-0 | `accountNumber`/`accountName` par une requête propre ; `letterable_account`, `group_account_number` inchangées |
| R L-3 | 15-1c-0 | colonnes ajoutées (la jointure existe) ; constante SQL et `struct` de ligne propres ; `find_group` inchangée |
| F-2 | 15-1c-0 | transaction de lecture (C-15-1c-26) |
| F-6 | 15-1c-0 | composition par champs communs, jamais `flatten` |
| R L-10 | 15-1c-0 | contrôle documentaire restreint au paragraphe du `GET` |
| F-8 | 15-1c-0 | Status : première passe tenue en P3 de l'ensemble |
| F-3 | 15-1c-i | `LETTERING_CONCURRENT_CHANGE` vide aussi la sélection, exception assumée et motivée (C-15-1c-24) |
| F-4 | 15-1c-i | numéro de la pièce d'un groupe `document` : facture ou facture fournisseur, sinon facture du règlement (C-15-1c-25) |
| F-7 | 15-1c-i, 15-1c-ii | garde `i18n-entrees-a-variables` nommée |
| R L-8 | 15-1c-i | compteur, champ « Code », ordre du serveur testés ; `letteringOrigin` retiré des champs lus |
| obs. F | 15-1c-i | numéro de compte E2E ≤ 10 caractères |
| R L-4 | 15-1c-ii | scénarios E2E 7 à 9 posent leur propre lettrage |
| R L-5 = F-1 | 15-1c-ii | pied *Total* de la fiche d'écriture, test avec et sans projets |
| R L-6, R L-9, F-9 | 15-1c-ii | 30 sites (24 + 5 + 1) ; « trois de logique et l'E2E » |
| R L-7 | 15-1c-ii | preuve négative élargie aux textes provisoires des 15-1a2-* |

**Propagation** (valeurs grepées sur les trois fiches et l'index) : `flatten` — seulement pour l'interdire ;
« nom se lit dans la même » — plus nulle part ; « 24 lignes » — remplacé par « 30 » ; `letteringOrigin` — seulement
pour dire qu'il n'est pas lu ; « validation P1 due » — plus nulle part.

**Recompte** : 15-1c-0 — 3 / 5 / 9 ; 15-1c-i — 12 / 9 / 12 ; 15-1c-ii — 6 / 7 / 6 (critères / tâches / tests),
inchangés.

**Verdict** : 0 au-dessus de LOW. Les remédiations de cette passe touchent la spécification de code de production
(15-1c-0 : transaction de lecture, requête du compte, `struct` de ligne) : **passe ciblée de fin de boucle due**
(Haiku, une lentille, braquée sur le seul commit de cette remédiation).

### Validation P2 — 2026-10-09 (Opus 5.5 ×2, lentilles R et F ; remédiation Opus 5.5, en autonomie) — SERVEUR EXTRAIT EN 15-1c-0

Prompt versionné : `15-1c-validate-prompt-p2.md`. Rapports : `/home/gcorbaz/devel/kesh-gate-logs/15-1c-validate-p2-{R,F}.md`.
**R : 0 HIGH / 5 MEDIUM / 11 LOW ; F : 0 HIGH / 6 MEDIUM / 10 LOW.** Recoupements : R M-1 = F2-1 ; R M-2 = F2-2 ;
R M-3 = F2-4 ; R M-5 = F2-5 ; R M-4 ⊃ F2-L5 ; R L-1 = F2-L2 ; R L-2 = F2-L4 ; R L-3 = F2-L3 ; R L-4 = F2-L9 ;
R L-5 ≈ F2-L7 ; R L-6 ≈ F2-L8 (part README) ; R L-7 = F2-L1 ; R L-9 ⊂ F2-3 → **7 MEDIUM distincts**, tous **nés de
la remédiation P1** (`bb894777`). **Signal D5 levé** (recyclage : défauts nés du correctif précédent) ; décision de
l'orchestrateur : extraire la partie serveur en **15-1c-0** (F2-3), remédier le reste dans la foulée. Chaque finding
vérifié au code de `f9b6b199` avant d'être appliqué. Décisions au registre : **C-15-1c-14 à C-15-1c-23**.

| finding | sév. | vérification | sort | où |
|---|---|---|---|---|
| F2-3 (+ R L-9) | MEDIUM | 15-1b : story serveur sans écran, livrable seule ; C-15-1a2-23 extrait la refonte du socle en story préalable : confirmé | **découpage** : 15-1c-0 = AC15, AC16, tests serveur ; motif de la dérogation de la 15-1c-i réécrit | 15-1c-0 ; 15-1c-i Dérogation ; C-15-1c-14 |
| R M-1 = F2-1 | MEDIUM | `kesh-db/src/errors.rs` `LetteringLineOwnedByDocument { blocker, document_id, document_label }` ; `kesh-api/src/errors.rs` `entry_document_refusal_response` → `refusal_409` (suffixe) ; `kesh-db/tests/letterings.rs:802-803`, `supplier_invoices_repository.rs:2568` ; lectures paresseuses `letterings.rs` (refus 2 seulement pour `Reversal`, borne après) : confirmé | **corrigé** : `ManualDissolutionBlocker` + `manual_dissolution_step` à faits optionnels (`NeedOwnership`, `NeedPeriod`) ; la dissolution lit comme aujourd'hui et rend l'erreur de `first_document_owner` inchangée ; test 5 : mêmes `details` | 15-1c-0 AC15, test 5 ; C-15-1c-15 |
| R M-2 = F2-2 | MEDIUM | 15-1b « Définitions » : « au signe près », `debit_sense` ; `kesh-report/src/opening.rs` `signed` : confirmé | **corrigé** : pied en valeur absolue + « débiteur » / « créditeur » (signe seul, aucun type lu) ; phrase « du côté naturel du compte » ; lien du Grand livre et manuel « au signe près » ; test Vitest sur un passif | 15-1c-i AC2, AC8, test 10 ; 15-1c-ii AC9, AC12 ; C-15-1c-16 |
| R M-3 = F2-4 | MEDIUM | `journal_entries.rs` étape 9 (lignes supprimées puis réinsérées) ; `POST` : 400 de forme (`LETTERING_TOO_FEW_LINES`, `…_TOO_MANY_LINES`), 404, 409 sinon (`kesh-api/src/errors.rs`, table des codes) : confirmé | **corrigé** : tout 404/409 du `POST` et du `DELETE` recharge liste et propositions et vide la sélection ; texte d'écran pour le 404 | 15-1c-i AC4, AC5, AC6, tests 7, 8, 9 ; C-15-1c-17 |
| R M-4 (⊃ F2-L5) | MEDIUM | 15-1a2-0 D5 réécrit `error-lettering-is-document` (« il suit la pièce et ses règlements ») ; avoir → groupe `document` (15-1a2-i) : confirmé | **corrigé** : « lettrage de la pièce » ; le texte de la clé non cité ; manuel « il suit la pièce et ses règlements » | 15-1c-i AC6, AC13, test 9 ; 15-1c-ii AC12 ; C-15-1c-18 |
| R M-5 = F2-5 | MEDIUM | `grep -n -i lettr docs/manual/fr/marketing-brochure.tex` → `:420`, bloc « Backlog (Epic 13 à 15) » : confirmé | **corrigé** : inventaire sur `docs/manual/fr/*.tex` ; brochure à AC17 | 15-1c-ii AC12, AC17, T5 ; C-15-1c-19 |
| F2-6 | MEDIUM | `GeneralLedgerView.svelte` : 8 en-têtes, `colspan` 7 + 1, 8, 5 + 3 : confirmé | **corrigé** : colonne en dernier ; test par somme des `colspan` de chaque `<tr>`, pied compris, débit/crédit à leur index | 15-1c-ii AC9, test 3 ; C-15-1c-20 |
| R L-1 = F2-L2 | LOW | `Origin` : trois valeurs | **corrigé** : douze combinaisons | 15-1c-0 test 5 |
| R L-2 = F2-L4 | LOW | `letterings_e2e.rs` : `l.get(champ).is_some()`, longueur des lignes au `GET` : confirmé | **corrigé** : test 8 **neuf**, ensembles exacts des clés | 15-1c-0 tests 1, 8 |
| R L-3 = F2-L3 | LOW | 15-1b AC2 : B = `document`, `documentState`, `amountDue`, `inOpenPeriod` ; `FIND_GROUP_SQL` joint déjà `journal_entries` : confirmé | **corrigé** : `journal`/`description` lus avec les lignes ; part « pièce et période » seule, sans reste dû | 15-1c-0 AC15 |
| R L-4 = F2-L9 | LOW | `fr-CH/messages.ftl:50, 53, 54` : une ligne chacune ; suffixe `refusal_409` : confirmé | **corrigé** : « texte de la clé », relevé au T0, suffixe non recomposé | 15-1c-i AC6 |
| R L-5 ≈ F2-L7 | LOW | PDF aplati : `pdftotext … \| grep -c ellemême` → 1 (césure) : confirmé | **corrigé** : preuve négative sur les `.tex`, `l.` pour les deux apostrophes ; PDF : régénération et titre | 15-1c-ii AC12 point 3 ; C-15-1c-23 |
| R L-6, F2-L8 | LOW | `README.md` ligne v0.13.0 ; règle d'inclusion : confirmé | **corrigé** : README par chaque story ; *Modifié* du `GET` écrit par la 15-1c-0 | 15-1c-0 AC18 ; 15-1c-i AC19 ; 15-1c-ii AC14, AC17 ; C-15-1c-21 |
| R L-7 = F2-L1 | LOW | `supplier_invoice_cancel_letters_a_pair_that_cannot_be_dissolved_by_hand` (`supplier_invoices_repository.rs:2511`) : confirmé | **corrigé** : geste réel, aucun SQL brut | 15-1c-0 test 3 |
| R L-8 | LOW | appelants de la dissolution hors 15-1a-i/ii | **corrigé** : inventaire `grep -rln` au T0 | 15-1c-0 T0 |
| R L-10 | LOW | 15-1c-ii : AC14, AC17 sans contrôle | **corrigé** : test 6 | 15-1c-ii |
| R L-11 | LOW | 15-1c-i : AC11 sans test | **corrigé** : test 12 | 15-1c-i |
| F2-L6 | LOW | 15-1a2-i : corps dans `dissolve_group_inner` | **corrigé** : relevé au T0 | 15-1c-0 T0, contrats |
| F2-L10 | LOW | composant de lien « s'il est partagé » | **corrigé** : créé par la 15-1c-i, sans clé | 15-1c-i AC11 ; C-15-1c-22 |

**Propagation post-patch** (valeurs grepées sur les fiches 15-1c-*, le registre, l'index 15-1, le README) :
« règlement de la pièce » — ne subsiste nulle part dans les fiches vivantes (15-1c-i AC6 et AC13 réécrits) ;
« annulez le règlement » / « annuler le règlement » — seulement dans la 15-1c-i, pour dire que la 15-1a2-0 l'a
retiré, et au registre (C-15-1a2-*, historique) ; `colspan` — seuls AC9 et le test 3 de la 15-1c-ii, cohérents ;
`marketing-brochure` — AC12, AC17, T5 de la 15-1c-ii ; `huit combinaisons` — plus nulle part ; « clés figées » —
remplacé par « ensemble exact » à la 15-1c-0 ; `AC15`/`AC16` dans la 15-1c-i — seulement comme renvois à la 15-1c-0 ;
`15-1c-i → 15-1c-ii` dans les chaînes d'ordre — toutes passées à `15-1c-0 → 15-1c-i → 15-1c-ii` (fiches 15-1c-*,
`15-1-lettrage.md`) ; fiches validées amont (15-1b, 15-1a2-*) laissées : elles parlent de « la 15-1c », que l'index
couvre. README (ligne v0.13.0) et `sprint-status.yaml` (clé `15-1c-0-groupe-de-lettrage-enrichi`) mis à jour.

**Recompte** (depuis les fichiers, `grep -c '^\*\*AC[0-9]'`, `grep -c '^- \[ \] \*\*T'`, `grep -cE '^[0-9]+\. AC'`) :
15-1c-0 — **3** critères, **5** tâches, **9** tests ; 15-1c-i — **12** critères, **9** tâches, **12** tests et
6 scénarios E2E ; 15-1c-ii — **6** critères, **7** tâches, **6** tests et 3 scénarios E2E.

**Verdict : validation P3 due** — passe complète (Sonnet), sur **les trois** sous-fiches ensemble.

### Validation P1 — 2026-10-09 (Sonnet 5.5 ×2, lentilles R et F ; remédiation Opus 5.5, en autonomie) — DÉCOUPÉE

Prompt versionné : `15-1c-validate-prompt-p1.md` (`8e36a146`). Rapports :
`/home/gcorbaz/devel/kesh-gate-logs/15-1c-validate-p1-{R,F}.md`. **R : 2 HIGH / 8 MEDIUM / 6 LOW ; F : 2 HIGH / 8
MEDIUM / 5 LOW.** Recoupements (orchestrateur) : R-1 = F-1 (partie Balance) ; R-2 = F-2 ; R-3, R-4, R-5 ⊂ F-1 ;
R-8 = F-9 ; R-10 = F-10 ; R-9 ≈ F-11 ; R-7 ≈ F-6 / F-14 ; R-5 (d) = F-3 ; R-11 = F-5 ; R-14 = F-15. Chaque finding
vérifié au code de `e892dcfa` (`grep -nF`, lectures) et dans les fiches validées avant d'être appliqué. Décisions au
registre : **C-15-1c-1 à C-15-1c-13**.

| finding | sév. | vérification | sort | où |
|---|---|---|---|---|
| R-1 = F-1 (Balance) | HIGH | 15-1b « Définitions » (« Ce solde EST celui de la Balance ») et point 1 : confirmé | **corrigé** : phrase réfutée retirée, la vraie écrite | 15-1c-i AC8 |
| R-2 = F-2 | HIGH | `routes/letterings.rs` `LetteringLineResponse` : 8 champs, ni pièce, ni journal, ni libellé, ni période : confirmé | **corrigé** : `GET /letterings/{key}` enrichi par réemploi de la requête B ; prévision du refus par la fonction pure que la dissolution appelle (C-15-1c-2) ; « aucune route neuve » tenu (route existante, réponse enrichie) | 15-1c-i AC15, AC16, AC6 |
| R-3 (⊂ F-1) | MEDIUM | 15-1b AC4 : deux champs, `nothingDue` sans libellé : confirmé | **corrigé** : AC3 en deux dimensions, six libellés, état d'aujourd'hui ; « par … » non affiché (aucun auteur au contrat) ; AC2 avec l'exercice | 15-1c-i AC2, AC3 |
| R-4 (⊂ F-1) | MEDIUM | 15-1b AC3, AC5 ; `MAX_LINES_PER_GROUP = 200` (`kesh-core/src/lettering.rs:27`) : confirmé | **corrigé** : case par `manuallyLetterable`, infobulle par cause, `inOpenPeriod`, sélection multi-pages, `total` des propositions, périmée → rechargement, `LETTERING_CONCURRENT_CHANGE` | 15-1c-i AC4, AC5 |
| R-5 (⊂ F-1), F-3 | MEDIUM | `grep -rn LETTERING_FISCAL_YEARS_CLOSED crates frontend/src docs` : aucune sortie ; C105 révise C94 (registre) : confirmé | **corrigé** : code réel ; règle C105 citée au manuel ; points 7–10 intégrés ; état d'URL `?group=` (C-15-1c-3) | 15-1c-i AC1, AC6 ; 15-1c-ii AC12 |
| R-6 | MEDIUM | `git log` : sur `e892dcfa`, seules 15-1a-i et 15-1a-ii livrées : confirmé | **corrigé** : ordre complet dans les deux Status et l'index ; E2E atteignables après la 15-1b, scénario (1) après la 15-1a2-i | Status, index |
| R-7 ≈ F-6 / F-14 | MEDIUM | (a) **en partie réfuté** : la page des rapports charge déjà les comptes **archivés compris** (`reports/+page.svelte:127`, `fetchAccounts(true)`) ; `LedgerSection` sans `letterable` : confirmé. (b) `ENTRY_LETTERED` câblé (`form-helpers.ts:120` → `stale`), refus de suppression en toast puis relecture (`[id]/+page.svelte`, `confirmDelete`) : confirmé. (c) exports : C-15-1b-6 | **corrigé** : lien depuis `GET /accounts` (C-15-1c-8), `asOf` = fin de période ; lien vers le groupe dans le **motif d'écran**, jamais dans le toast (C-15-1c-9) ; exports **exclus**, dit au manuel | 15-1c-ii AC9, AC12 |
| R-8 = F-9 | MEDIUM | `grep -n 'subsection{' user-manual.tex` : *Modifier ou supprimer* `:474`, contre-passation `:601`, règlement `:1141`, avoirs `:1259`, fournisseurs `:1400`, Grand livre `:1870`, glossaire *Lettrage* `:2431-2437` ; `grep -c -i lettr` = 24 ; `CHANGELOG.md:15`, `api-external.md:291`, `README.md:222` : confirmé | **corrigé** : propagation **par inventaire** (commandes au T0, chaque site classé), sites désignés par section et phrase ; CHANGELOG, `api-external.md`, README, site couverts | 15-1c-ii AC12, AC14, AC17 |
| R-9 ≈ F-11 | MEDIUM | `tests/e2e/helpers/api-fixtures.ts` : aucun helper de compte ni de règlement ; seed : 1000/1100/2000/3000/4000 : confirmé. ⚠️ « les lignes des autres specs polluent » **en partie réfuté** : les specs repartent d'un preset (`seedTestState`, troncature) — le remède est gardé, pour l'isolement et parce que les écritures lettrées restent figées | **corrigé** : tests numérotés par AC (Rust, Vitest, E2E) dans chaque sous-fiche ; compte lettrable **créé** par le spec, montants uniques ; rôle Consultation en E2E (C-15-1c-13) | 15-1c-i T9, AC13 ; 15-1c-ii T6, AC13 |
| R-10 = F-10 | MEDIUM | recompte au grain fin : > 5 | **découpage** (C-15-1c-1) ; dérogations au grain fin (C-15-1c-11) | index, Dev Notes |
| F-4 | MEDIUM | `accounts.api.ts:10` `includeArchived = false` par défaut ; C96 « un compte archivé reste lettrable » : confirmé | **corrigé** : `fetchAccounts(true)`, archivés marqués | 15-1c-i AC1 |
| F-5 = R-11 | MEDIUM / LOW | `bank-import/[id]` prend un identifiant d'import ; aucune route de transaction : confirmé | **corrigé** : table type → cible ; `bankTransaction` sans lien ; `settlement` sans lien si `invoiceId` nul (C-15-1c-6) | 15-1c-i AC2 |
| F-7 | MEDIUM | `balance.ts` (`big.js`) ; plafond 200 : confirmé | **corrigé** (avec R-4) : décimal, 200, pages (C-15-1c-4) | 15-1c-i AC4 |
| F-8 | MEDIUM | 15-1b AC5 « aujourd'hui », pas d'`offset` : confirmé | **corrigé** : chargement séparé, échec indépendant, indépendant de `asOf`, rechargement après acceptation ou refus périmé (C-15-1c-7) | 15-1c-i AC5 |
| F-12 | LOW | `+layout.svelte:109-110` ; `i18n-keys.test.ts:526-529` (`sitesTotal: 1920`) : confirmé | **corrigé** : aucun numéro de ligne prescrit ; bornes relevées aux deux bornes | 15-1c-i AC11 |
| F-13 | LOW | ⚠️ **référence fausse** : C132 porte sur la dévalidation ; le vocabulaire est **C-15-1a-i-4** et **C-15-1a-ii-2** (`grep -n Ausgleich epic-15-choix-autonomes.md`). En-têtes en dur de la fiche (`Compte`, `Débit`) : confirmé | **corrigé** : vocabulaire imposé (référence juste) ; gardes nommées (G4-bis, G8/G8-bis, G9, G13, parité, un repli par clé, libellé en dur, sélecteurs traduits, bornes) ; gate backend dû ; en-tête « Lettrage » traduit | 15-1c-i AC11 ; 15-1c-ii AC9, AC11 |
| F-14 | LOW | voir R-7 (b) | **corrigé** (C-15-1c-9) | 15-1c-ii AC9 |
| F-15 = R-14 | LOW | 15-1b AC1 : `asOf` absent = date UTC du serveur : confirmé | **corrigé** : `asOf` toujours explicite, date locale, écrite dans l'URL (C-15-1c-5) ; lien du Grand livre à la fin de période | 15-1c-i AC1 ; 15-1c-ii AC9 |
| R-12 | LOW | voir F-12 ; replis et gardes | **corrigé** | 15-1c-i AC11 |
| R-13 | LOW | `CHANGELOG.md:15` titre « par l'API » : confirmé | **corrigé** : titre réécrit, phrase « L'écran viendra » retirée ; glossaire | 15-1c-ii AC14, AC12 |
| R-15 | LOW | `parse_reference` → 404 indiscernable | **corrigé** : « aucun groupe ne porte ce code » | 15-1c-i AC6 |
| R-16 | LOW | #518 : compte fournisseur | **non retenu**, motif écrit : l'écran ne dépend pas du type de pièce (il lit `document`, `manuallyLetterable`) ; la vue sur les pièces fournisseurs est prouvée côté serveur par la 15-1b (tests 6, 7 : facture fournisseur, achat et règlement) ; un E2E fournisseur exigerait un helper de paiement absent pour une assertion que le serveur porte déjà | — |

**Propagation post-patch** (valeurs grepées sur tout le dépôt) : `LETTERING_FISCAL_YEARS_CLOSED` — il
n'apparaissait, hors des textes historiques, que dans le corps remplacé de cette fiche (retiré) ; la 15-1a-socle
et la 15-1a-i (`:530`, « retiré ») le citent comme le code d'avant C113 ; le registre aussi — inchangés, ils sont historiques. Dans la 15-1c-i, il n'apparaît que pour dire qu'il n'existe pas. « ne lit que ses écritures » — ne subsiste qu'au point 1 de « Pour la 15-1c » de la 15-1b (qui la déclare fausse) et dans la 15-1c-i AC8 (qui la cite pour la retirer). `C94` — la 15-1c-ii cite C105 qui le révise.
`15-1c-proposition-ecran` — renvois des fiches validées (lecture seule) laissés : ils désignent désormais l'index,
qui pointe vers les deux sous-fiches. Index `15-1-lettrage.md`, `sprint-status.yaml` (clés `15-1c-i-ecran-postes-ouverts`,
`15-1c-ii-lettrage-dans-kesh`, la clé `15-1c-proposition-ecran` passée à `split`) et `README.md` (ligne v0.13.0)
mis à jour.

**Recompte** (depuis les fichiers, `grep -c '^\*\*AC[0-9]'`, `grep -c '^- \[ \] \*\*T'`, `grep -cE '^[0-9]+\. AC'`) :
15-1c-i — **13** critères, **10** tâches, **19** tests numérotés (8 Rust, 11 Vitest) et 6 scénarios E2E ;
15-1c-ii — **6** critères, **7** tâches, **5** tests numérotés et 3 scénarios E2E. Le corps d'avant (14 critères,
7 tâches) n'est plus dans ce fichier.

**Verdict : validation P2 due** — passe complète (Opus), sur **les deux** sous-fiches ensemble : deux HIGH corrigés,
un enrichissement serveur neuf (AC15) qui touche `dissolve_group_in_tx`, et un découpage dont la couture se relit.

### Corps d'avant le découpage — résumé

Le corps remplacé (Story, Reprise du 2026-10-08, Reçu de la 15-1a points 1–11, AC1–AC14, T1–T7, Dev Notes) se lit à
`8e36a146`. Les entrées ci-dessous décrivent ce corps et ceux qui l'ont précédé.

### Reçu de la validation P3 du socle — 2026-10-09 (Opus 5.5, remédiation de la 15-1a)

Section « Reçu de la 15-1a » complétée des points 6 à 11 (registre C124, C126, C127) : ordre avec la
15-1a découpée ; le manuel de la modification (sixième condition, réserves) déjà touché par la
15-1a-ii ; `ENTRY_LETTERED` en dernier ; glossaire à `:2323` ; rubriques du CHANGELOG ; exercice par
ligne dans les routes. Dépendance de tête mise à jour. Corps non réécrit.

### Reprise du 2026-10-08 — réécriture contre le modèle réel (Opus 5.5, en autonomie)

Corps réécrit (registre C95, C99). Le moteur de proposition passe en 15-1b ; la fiche devient
l'écran seul, avec l'emplacement tranché (Q4 du dégel). Les décisions ouvertes (3) et (4) de la
passe 3 d'août sont closes (tête de fiche). **14 critères** (AC1–AC14), **7 tâches** (T1–T7),
recomptés depuis ce fichier.

*Entrées antérieures à la reprise — le corps qu'elles décrivent a été remplacé :*


### Passe 4 de `validate` — 2026-08-26 (Sonnet, contexte frais)

**1 HIGH, 2 MEDIUM, 1 LOW.** Sévérité décroissante (`2 HIGH` → `1 HIGH`).

⛔ **P4-1 (HIGH) — l'arbitrage affirmait « le même ensemble », et la décision ouverte (2) du
MÊME document le contredisait trois paragraphes plus loin.** Un écart structurel subsistait :
15-1b borne sa vue aux comptes `Receivable`/`Payable` **dans la requête** ; le moteur n'avait
**aucune** borne de rôle. On pouvait donc pointer le moteur sur le **compte bancaire ledger** —
que la vue n'ouvrira jamais — et y apparier des lignes que la réconciliation gère par un tout
autre mécanisme. 15-1c y répondait par **un bandeau** : **un bandeau ne protège pas l'API**.

→ **AC8** : le moteur ne travaille que sur les comptes que la vue ouvre, même borne, dans la
requête. ⚠️ Sur **`singleton_role`**, jamais sur `role` — leçon déjà payée par 15-1b, dont la
colonne générée vaut `NULL` pour un compte archivé.

✅ **L'alignement des deux fiches est désormais RÉEL**, et non plus seulement proclamé.

**P4-2 (MEDIUM) — résidu de propagation** : T2 prescrivait encore l'inscription au Pattern 5,
qu'AC6 venait de déclarer **non applicable** en passe 3. Corriger le critère sans greper la
tâche : le geste même que la § *Propagation post-patch* décrit, et la troisième fois qu'il se
produit dans cet epic.

**P4-4 (MEDIUM)** — **AC4-bis n'avait ni tâche ni test** depuis sa création en passe 3, alors
que son propre texte en exigeait un. Sans le tri, le plafond tronque un ensemble non ordonné
et évince **en premier** la contre-passation que la Réserve 2 protège — le critère introduit
pour fermer ce défaut serait resté non implémenté.

**P4-3 (LOW)** — le décompte de la passe 3 ne se recomptait pas : « quatre MEDIUM laissés » n'en
nommait que **trois**, et P3-9/P3-10 relèvent des LOW. Rectifié.

**Réfuté** : la borne de performance ne se dégrade **pas** avec l'alignement — l'absence de
prédicat réducteur était acquise avant l'arbitrage ; l'index composite est bien livré par le
socle ; et le paragraphe « ce que l'arbitrage supprime » décrit des findings de **15-1b**, ce
qui est défendable pour un texte partagé entre fiches sœurs.

La spec passe de **9 à 10 critères**. **Verdict : passe 5 due** — un HIGH au rapport, corrigé
ici.

### Arbitrage du Project Lead — 2026-08-26 : « ouvert » = « non lettré »

⛔ **La CINQUIÈME décision — celle que ni 15-1b ni 15-1c ne portait — est tranchée.** Les deux
passes 3, indépendantes, avaient conclu qu'elle décidait de la forme de la requête que les
deux stories allaient écrire.

> **« Ouvert » signifie « non lettré ». La vue ne regarde ni `paid_at`, ni aucun statut de
> facture, et ne joint aucune table de factures.**

✅ **Ce que l'arbitrage achète — un INVARIANT, pas une commodité.** Toute paire lettrée se
nettant exactement à zéro (15-1a AC4 et AC12), **la somme algébrique des lignes ouvertes d'un
compte égale son solde**, sans exception. C'est **testable en une assertion**, et c'est ce qui
rend enfin vraie la promesse du *so that* : *« justifier le solde d'un compte »*. Aucune des
deux définitions concurrentes ne le permettait.

⛔ **Ce qu'il coûte, et qui doit être assumé à l'écran** : une facture réglée par virement
importé réapparaît **ouverte** tant qu'elle n'est pas lettrée. C'est **comptablement vrai** —
la réconciliation ne crée aucune écriture, le compte porte toujours son débit — mais
contre-intuitif. **AC4 de 15-1b devient de ce fait le critère le plus important de la fiche**,
et il doit offrir un chemin vers le lettrage, pas seulement une explication.

✅ **Ce qu'il SUPPRIME, et c'est le plus notable** : **trois HIGH et trois MEDIUM des passes 1
à 3 tombent avec lui** — la contradiction entre fiches sœurs, la déclinaison en trois puis
quatre cas, les deux tables de factures, les deux écritures fournisseur, la troisième écriture
d'annulation non référencée. **Ce n'est pas une simplification cosmétique : c'est la
disparition de la classe entière de défauts que ces passes trouvaient**, tous nés de ce que la
vue tentait de concilier deux mécanismes que rien n'oblige à concilier.

⚠️ **Ce qu'il NE tranche PAS.** La Décision 1 (relance) reste ouverte et son enjeu se
**déplace** : la vue ne lit plus `paid_at`, mais les **cinq lecteurs** recensés continuent de
le lire — et le plus grave est comptable, pas cosmétique. `reconciliation.rs` proposera une
facture lettrée mais non marquée payée à un **second règlement** : **soldée deux fois**, une
fois en caisse et une fois en banque. La Décision 3 reste ouverte pour la même raison.

### Passe 3 de `validate` — 2026-08-25 (Opus, contexte frais)

⛔ **2 HIGH, 6 MEDIUM, 4 LOW. LA SÉVÉRITÉ REMONTE** — `2 HIGH+2 MED` → `2 MED+1 LOW` →
`2 HIGH+6 MED`. **Le critère de non-convergence de la § *Règle de splitting préventif* est
déclenché**, pour la seconde fois dans cet epic.

⚠️ **Mais le diagnostic n'est PAS « la fiche est trop large » — elle est trop COUPLÉE à ses
sœurs.** Cinq des huit findings > LOW sont des **coutures** entre 15-1a, 15-1b et 15-1c, et
deux n'existent que depuis la passe 2 de **15-1b**, tombée pendant cette revue. **Une passe 4
sur 15-1c seule ne verrait pas la suivante.** Ce qu'il faut n'est pas une passe de plus, mais
**une relecture des trois fiches ENSEMBLE sur la seule question « qu'est-ce qui est ouvert, et
qui le dit ».**

⛔ **P3-1 (HIGH) — 15-1c et 15-1b prescrivent, chacune par un test nommé, deux comportements
INCOMPATIBLES.** Sur le compte fournisseur, AC2 exige que la paire soit **proposée** ;
AC3-bis de 15-1b exige que le compte n'affiche **aucune ligne ouverte**. Le même écran dirait
« 0 ligne ouverte » et « 1 rapprochement proposé » sur **les mêmes deux lignes**. → **décision
ouverte**, c'est un arbitrage de produit.

⛔ **P3-2 (HIGH) — AC7 ne bornait pas ce qu'il disait borner.** Ses deux bornes agissaient
**après** l'appariement ; le `LIMIT 50` qu'il citait en modèle borne le **jeu candidat**, dans
un `WHERE` que la fenêtre **et** la tolérance réduisent. Or la passe 2 a retiré la fenêtre du
filtre et AC2 interdit `status`/`paid_at` : **il ne restait aucun prédicat réducteur**. Et
l'analogie était fausse — la réconciliation est **1 → N**, le lettrage **N → N**. → borne sur
le **jeu candidat**, plafond **chiffré à 500**, et la remarque que l'égalité stricte rend
l'appariement **groupable par montant**, donc linéaire.

**Trois MEDIUM corrigés ici** : **P3-3** le « classement » tranché en passe 2 n'existait dans
aucun critère — le plafond tronquait un ensemble non ordonné, évinçant **en premier** la
contre-passation que la Réserve 2 protège → **AC4-bis** ; **P3-4** la fenêtre figurait
**toujours** comme critère d'éligibilité dans le tableau, la passe 2 ayant corrigé les deux
sites qui en *parlent* et laissé les deux qui la *prescrivent* — le geste même que la
§ *Propagation post-patch* codifie ; **P3-6** ⚠️ **mon affirmation « ce document exige que tout
nouvel endpoint y figure » était FAUSSE** — le Pattern 5 impose un ordre à ceux qui prennent
**plus d'un verrou**, et les routes de 15-1c sont des routes de **lecture** ; **P3-9/P3-10** le
décompte d'index (trois, pas deux) et le titre du tableau, que sa propre colonne contredisait.

**Trois MEDIUM laissés en décisions ouvertes** *(rectifié en passe 4 : « quatre » n'en nommait que trois, et P3-9/P3-10 sont deux des quatre LOW, pas des MEDIUM corrigés)* — voir le § dédié : la borne de rôle du moteur,
le contrat d'acceptation et le pattern de lot, AC4 sans tâche ni test.

**Réfuté** : le déplacement de l'index vers 15-1a est **correct des deux côtés**, `CREATE INDEX`
n'impose aucun bump `min_required`, et la colonne « provenance » est exacte sur ses trois lignes
NEUF.

**Verdict : relecture croisée des trois fiches due, pas une passe 4 sur celle-ci.**

### Passe 2 de `validate` — 2026-08-25 (Haiku, contexte frais)

**0 HIGH, 2 MEDIUM, 1 LOW.** Sévérité décroissante (`HIGH → MEDIUM`) : convergence monotone.

**Aucune régression des patches de la passe 1** : AC6, AC7, la colonne « provenance » et le
critère `lettering_id IS NULL` sont vérifiés exacts au sol et jugés bien posés.

⚠️ **Les deux MEDIUM ne sont pas des défauts de raisonnement mais des ambiguïtés de
COORDINATION entre fiches** — la classe d'erreur que le split fabrique, et la troisième fois
qu'elle se manifeste dans cet epic.

⛔ **P2-1 — AC7 exigeait un index composite que PERSONNE ne créait.** 15-1a ne posait qu'un
`idx_jel_lettering (lettering_id)` simple ; le composite `(account_id, lettering_id)` n'était
la tâche d'aucune des deux fiches. → **Tranché : la migration vit dans le socle**, donc
`idx_jel_account_lettering` a été **ajouté au DDL de 15-1a** ; 15-1c le **consomme**.

⚠️ **Et les deux index sont nécessaires**, le préfixe gauche ne permettant pas de les
confondre : `(lettering_id)` sert « retrouver la contrepartie d'une marque »,
`(account_id, lettering_id)` sert « les lignes ouvertes du compte A » — la requête du moteur
**et** celle de la vue de 15-1b.

⛔ **P2-2 — « tranchée par défaut » ne disait pas LAQUELLE des deux conduites était le défaut,
et AC2 en dépendait.** → **Tranché : la fenêtre s'applique au CLASSEMENT, pas au filtre.**
C'est la seule lecture cohérente avec AC2, qui exige un test datant les deux pièces à **plus de
30 jours d'écart** : si la fenêtre filtrait, ce test **échouerait par construction**, et deux
développeurs auraient raison en même temps — celui qui écrit le moteur avec un filtre, celui
qui écrit le test tel qu'AC2 le prescrit. **C'est la contradiction « proposer ce qu'on refuse »
de la Réserve 1, transposée à la date.**

**P2-3 (LOW)** : « inscrire au tableau du Pattern 5 » ne disait ni quoi ni sous quelle forme —
le format des lignes voisines est désormais nommé.

**Vérifié et réfuté** : multi-devise et arrondis `DECIMAL(19,4)` sans objet ; AC1 et AC2
complémentaires, pas antagonistes ; la Réserve 1 correctement posée, sa conséquence énoncée.

**Verdict : passe 3 due** — deux MEDIUM au rapport. ⚠️ Aucun ne relève d'un arbitrage produit :
les deux sont tranchés ici sur la cohérence interne, et **restent réversibles d'un mot**.

### Passe 1 de `validate` — 2026-08-25 (Sonnet, contexte frais)

**2 HIGH, 2 MEDIUM.** Tous vérifiés au sol par l'orchestrateur avant application.

⚠️ **Les quatre findings répètent le motif des huit passes de 15-1a** : une lecture de la spec
contre elle-même les aurait tous laissés passer. **Seule la lecture des chemins de code de
l'Epic 8 et du schéma de `journal_entry_lines` les révèle.**

| | défaut | gravité | remède |
|---|---|---|---|
| **P1-2** | **Aucune mention de scoping multi-tenant** — zéro occurrence de `company_id`, « multi-tenant » ou « IDOR ». Or 15-1c ouvre une **surface d'API neuve** par un chemin **distinct** des routes que 15-1a a scopées, et `journal_entry_lines` n'a **aucun** `company_id` | **HIGH** | **AC6** — jointure, 404 indiscernable, inscription au Pattern 5 |
| **P1-4** | **Aucune borne de performance ni anti-DoS** — T1 et T2 tenaient en une phrase. La réconciliation en porte **trois** pour le même calcul : `LIMIT 50`, `MAX_PROPOSALS_LIMIT = 500` (*« défense anti-DoS contre `?limit=999999` »*) et un index dédié | **HIGH** | **AC7** — portée par compte, plafond serveur, composite `(account_id, lettering_id)` |
| **P1-1** | **La provenance de deux critères sur quatre était FAUSSE** : « même compte » et « sens opposés » **n'existent nulle part** dans l'Epic 8, dont le scoring est `0,50 montant + 0,40 référence + 0,10 contact`. Pire, `rules.rs` a **délibérément retiré** le seul filtre de sens qui ait existé | MEDIUM | colonne « provenance » au tableau ; T1 dit ce qui s'extrait et ce qui est neuf |
| **P1-3** | **Le critère `lettering_id IS NULL` manquait** au tableau **et** aux tests. AC10 de 15-1a protège la route, mais **rien n'empêchait le MOTEUR de re-proposer** une paire déjà lettrée | MEDIUM | critère ajouté, test nommé |

⛔ **P1-4 est aggravé par deux traits propres à cette story**, et c'est ce qui le rend HIGH
plutôt que MEDIUM : la **Réserve 2** envisage de retirer la fenêtre de dates du filtre — donc
de comparer toutes les lignes d'un compte sans borne temporelle — et **AC2 interdit de filtrer
sur le statut de facture**, si bien que l'ensemble candidat ne peut plus être réduit comme le
fait la réconciliation. Sur un compte fournisseur actif depuis plusieurs exercices — que **D3
autorise explicitement** —, l'appariement devient **quadratique et non plafonné**.

⚠️ **P1-3 dit quelque chose du découpage** : le socle protège la **route**, la story de l'écran
alimente le **moteur**, et le critère qui les relie n'était écrit ni dans l'une ni dans l'autre.
C'est la même classe de trou que l'égalité des montants, tombée entre 15-1a et 15-1c et
rapatriée en passe 3.

**Six pistes réfutées au sol**, dont : la tolérance de 5 centimes n'est **pas** justifiée par
les frais bancaires *dans le code* — ce narratif vient de la story mère, le commentaire réel
dit seulement *« réduit le candidate set sans accepter le mismatch »* ; le montant TTC/HT ne
pose **pas** ici le problème qu'il a posé à la réconciliation (#246), les lignes de grand livre
portant déjà le TTC ; la paire facture/avoir **est** structurellement exacte, l'avoir créditant
`total_ht + total_vat` au même compte ; et le règlement fournisseur crée bien une écriture au
même compte, sens opposé, même TTC, **pour les deux modes de règlement**.

La spec passe de **5 à 7 critères**. **Verdict : passe 2 due.**

### Création par split de la 15-1 — 2026-08-25

Issue du **split de la Story 15-1**. Recueille la correction majeure de la passe 1 sur D5
(les filtres de facture qui excluaient trois cas sur quatre) et le MEDIUM **P3-7** de la
passe 3 (la fenêtre de 30 jours qui tue la contre-passation).

⛔ **Deux réserves restent ouvertes** : la tolérance de montant face aux frais bancaires, et
la fenêtre de dates. Toutes deux sont tranchées **par défaut** dans la spec, avec leur
conduite alternative nommée — elles se changent d'un mot tant que le développement n'a pas
commencé.
