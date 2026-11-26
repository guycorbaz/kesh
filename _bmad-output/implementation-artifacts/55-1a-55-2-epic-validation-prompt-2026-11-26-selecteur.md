# Prompt — passe ciblée sur le sélecteur d'exercice (D15), 55-1a, 55-2 et fichier d'epic

Worktree `<worktree>/kesh-e55`, branche `epic/55-navigation`. Passe **ciblée**, contexte
frais, modèle Sonnet, en français. Les fiches sont au **plafond de huit passes** : cette passe ne compte pour aucune ; ses
MEDIUM et au-dessus vont au **T0** (R-n), sans remédiation. Elle juge **un seul commit**, celui dont le message commence par
« docs(epic-55) : sélecteur d'exercice des Rapports et du Grand livre versé (D15 » (`git log --oneline -3`, puis
`git show <ce commit>`).

## La décision à verser (référence ; décision de l'utilisateur et avis UX, à confirmer par le mainteneur)

- Le libellé « Exercice » une seule fois, hors du menu, par un vrai `<label for>` associé au `<select id>` ; l'`aria-label`
  en double retiré ; un lecteur d'écran lit « Exercice, liste déroulante, 2025 ».
- Le menu : `<select>` natif, valeur fermée en gras (`select { font-weight: 600 }`, `option { font-weight: 400 }`) ; il
  affiche les années (« 2025 », « 2025/2026 ») quand `name == nom_par_defaut(début, fin)`
  (`crates/kesh-core/src/accounting/exercice.rs:33-38`), sinon le nom saisi ; en cas de doublon de texte, les bornes,
  ajoutées aux seules entrées concernées : « 2025 (01.01–30.06) ». Le menu n'affiche plus les dates.
- Choisir un exercice remplit « Du » et « Au » ; dates toujours remplies à l'ouverture.
- Dates changées à la main : même exercice → le menu le montre, même partiel, et un lien « Exercice entier » ; exercices
  différents → option désactivée « Plusieurs » (DE « Mehrere », IT « Diversi », EN « Multiple ») ; une date hors de tout
  exercice → « Hors exercice » (« Ausserhalb », « Fuori esercizio », « Outside fiscal years ») ; début après fin → menu
  inchangé ; l'option désactivée disparaît une fois un exercice choisi.
- Le même composant `ReportSelector` sur tous les onglets, Grand livre compris ; « Du » / « Au ».
- Rapport périmé grisé, « Paramètres modifiés — cliquez sur Générer ».
- En-tête : « Exercice 2025 — du 01.01.2025 au 31.12.2025 », « du … au … (partie de l'exercice 2025/2026) », « du … au …
  (exercices 2024/2025 et 2025/2026) ».
- Remplace l'AC-14 de la 55-1a (gras partiel par `ui/select`, R-7 de la 55-1a, R-3 de la 55-2) et l'entrée « — dates
  libres » de la 55-2.

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

- **A1 — Fidélité** : chaque point de la décision est-il un critère vérifiable, ni plus ni moins ? Ce que les fiches
  ajoutent (C-55-1-8 *Générer* désactivé, exports désactivés quand périmé, compte du Grand livre dans le périmé, en-tête
  « hors exercice », doublon à bornes complètes) est-il déclaré comme choix des fiches ?
- **A2 — Prémisses au code**, ligne lue recopiée : `frontend/src/lib/features/reports/ReportSelector.svelte` (lignes
  citées 51, 55, 63, 75) ; `frontend/src/routes/(app)/reports/+page.svelte` (P17 `:151-173`, effet du compte analytique
  `:182-195`, requête par projet `:222-232`, bandeau du Grand livre `:638-714`, menu des rapports par compte analytique
  `:747-760`) ; les six vues et leur ligne « Période » ; `crates/kesh-api/src/routes/reports.rs` (`fiscal_year_id`) ;
  `crates/kesh-core/src/accounting/exercice.rs:33-38` ; clés `reports-ledger-from`, `reports-ledger-to` dans les quatre
  `messages.ftl`.
- **A3 — Cohérence interne** : le modèle de la 55-1a (P17, *Appliquer une URL* point 2, AC-5, AC-8 b', AC-9, clés affichée
  et en vol, mutations) contredit-il encore D15 quelque part ? L'URL (`fy` déduit des dates) tient-elle avec D1 et D5 ? La
  55-2 (AC-11 à AC-15, AC-9, tests, T3bis, note de version) et le fichier d'epic (Objectif, D15, Stories, Fichiers communs,
  Questions) disent-ils la même chose ? Restes de l'ancienne règle : `grep -n "dates libres\|ui/select\|font-semibold\|fiscalYearMatching\|R-5\|R-7"` sur les trois fichiers, chaque occurrence triée (historique daté ou résidu).
- **A4 — Constatable** : chaque critère dit-il comment on le constate (Vitest, mutation, E2E, capture) ? Un test vrai par
  construction est un finding.
- **A5 — Règles du projet** : i18n aux quatre locales sans concaténation, DRY (un seul module de règles, un seul
  composant), taille et règle de splitting (cinq modules), aucun mot-clé de fermeture, Change Logs datés, la question au
  mainteneur présente.

Sévérités : CRITICAL / HIGH / MEDIUM / LOW.

## Interdits

Lecture seule. N'écris aucun fichier du dépôt ni du worktree, sauf le rapport ci-dessous. Aucun `cargo`, `npm`, `node`,
`make`, `docker`, `mariadb`, aucun `scripts/*` (dont `scripts/prepare-release.sh`), **aucun `gh`**, aucun `git` qui modifie
(ni `commit`, ni `stash`, ni `checkout`), aucun port ni service. `grep`, `git show`, `git log`, `sed -n`, `cat` admis.

## Rapport

Un seul fichier : `$GATE_LOGS/s55-selecteur/selecteur-2026-11-26-validation.md` — verdict chiffré
**par fiche** (55-1a, 55-2, fichier d'epic) au format C/H/M/L, findings (identifiant `SE-n`, fiche, sévérité, preuve —
ligne lue —, correctif proposé), axes exercés et non exercés. Puis le même contenu en réponse.
