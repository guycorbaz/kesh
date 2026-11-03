# Prompt — validation P4 **ciblée** de la sous-fiche v016-2b (#816), une lentille, Opus

Tu fais une **passe ciblée** (`CLAUDE.md` § *La passe ciblée*) sur la sous-fiche `v016-2b-releve-manuel-ecran.md` de Kesh (comptabilité suisse,
Rust/Axum, SvelteKit, MariaDB 10.11) : la saisie manuelle d'un relevé bancaire (#816), découpée en `v016-2a` (backend)
et `v016-2b` (écran), release 0.17.0. **Une seule lentille**, braquée sur **le seul diff de la remédiation P3** — non sur
la story entière, déjà passée sous trois validations. Modèle : **Opus**. **Contexte frais**. Langue : **français**.

Worktree `/home/gcorbaz/devel/kesh-0162`, branche `story/v016-2-releve-manuel` ; code et manuels de `c5101b07` (la branche
ne touche que `_bmad-output/`).

## À lire

1. Le diff de la remédiation P3 : `git log --oneline -10`, puis
   `git show <commit « remédiation de la validation P3 des sous-fiches 2a et 2b »> -- _bmad-output/` — **toutes** les
   lignes ajoutées ou récrites de `v016-2b-releve-manuel-ecran.md`, de la parente `v016-2-releve-manuel.md` et du registre
   `v016-2-choix-autonomes.md`, et celles de `v016-2a-releve-manuel-backend.md` qui touchent la frontière (axe 2).
2. Les rapports P3 qui l'ont motivé : `/home/gcorbaz/devel/kesh-gate-logs/v0162b-validate-p3-R.md` et `-F.md`, et
   l'entrée « Validation P3 » du Change Log de la sous-fiche (traitement de chaque finding, LOW non pris).
3. Le code que le diff cite, ligne par ligne (`sed -n`, `grep -nF`).

## Axes (tous obligatoires)

1. **Le diff P3 de cette sous-fiche.** Chaque passage ajouté ou récrit : chaque `fichier:ligne` dit-il ce qu'on lui fait
   dire ? Chaque remède tient-il au code, sans contredire un autre AC de la sous-fiche, de l'autre sous-fiche ou de la
   parente ? Chaque finding déclaré « appliqué » l'est-il **entièrement** (cherche les copies du symptôme : la valeur, pas
   la formulation — `grep -n` de chaque nombre et de chaque chemin corrigé dans les trois fiches et le registre) ? Une
   entrée passée du Change Log a-t-elle été réécrite (interdit) ? Le « LOW non pris » est-il motivé ?
2. **Cohérence 2a ↔ 2b sur les `reason`.** Le test croisé prescrit à l'AC-24 (un test Rust qui lit la table TypeScript et
   l'égale à `ManualStatementReason::ALL` par `code()`) est-il réalisable avec ce que la 2a prescrit (AC-4 de la 2a :
   noms, `pub`, `ALL`) et décidable (forme de la table TypeScript à imposer pour qu'une extraction par texte soit sûre :
   clés littérales, un seul objet) ? Le Vitest de couverture et le test croisé ne se contredisent-ils pas ? La famille
   `bank-import-manual-reason-` et les compteurs `sitesGabarit`, `SITES_GABARIT_ATTENDUS`, `CANDIDATES_ATTENDUES` sont-ils
   prescrits justes (`frontend/src/lib/shared/i18n-keys.test.ts:773`, `:960`, `i18n-libelle-en-dur.test.ts:250`) ? M30 ?
3. **Cohérence 2a ↔ 2b sur l'extracteur.** L'écran traite-t-il le `400 VALIDATION_ERROR` sans `field` (corps rejeté) autrement
   qu'un `BANK_STATEMENT_INVALID` (AC-21, AC-22) ? Les règles client (« *Enregistrer* inactif », ordre de la raison,
   *Vérifier* actif, asymétries écrites) restent-elles compatibles avec les ensembles fermés et la limite de corps de la 2a ?
   Et, sur les textes : les clés réemployées (R4) existent-elles aux quatre catalogues
   (`grep -n "bank-import-labels-period\|common-loading" crates/kesh-i18n/locales/*/messages.ftl`) ? Les quatre textes
   d'interface « importée » (`messages.ftl:166`, `:1433`, `:1439`, `:1473`) ont-ils un propriétaire unique (aucun ne
   figure-t-il aussi à la 2a) ? Les aides E2E prescrites (`createUser` extraite, IBAN MOD-97, `isPrimary`, dates locales)
   tiennent-elles avec `helpers/test-state.ts` et `seedTestState('with-company')` ?
4. **Ce que la remédiation n'a pas touché et qu'elle aurait dû** : une phrase ancienne de la sous-fiche ou de la parente
   que le remède rend fausse (par exemple une mention de `415` ou `413`, de « `make fr` », d'une liste recopiée « sur le
   patron de `CSV_LINE_ERROR_LABELS` » qui ferait foi seule, de « montants uniques »).

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, à `/home/gcorbaz/devel/kesh-gate-logs/v0162b-validate-p4.md`. Il
contient :

1. **Les findings**, numérotés `P1`, `P2`…, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), le lieu
   (`fichier:ligne`), **la ligne lue citée** (sortie `grep -nF` ou extrait copié) et le remède proposé. Une affirmation de
   présence ou d'absence sans sortie `grep -nF` copiée ne compte pas.
2. **Les axes exercés** et **les axes non exercés**, un par un, avec ce que tu as lu ou cherché. Un « 0 finding » sans
   cette liste ne clôt rien et ne compte pas comme passe.
3. Un décompte final par sévérité, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture**, hors de ton seul rapport : ni `Edit`, ni `Write` sur un autre fichier, ni redirection (`>`, `>>`,
  `tee`) — pas même vers un fichier « jetable », ni dans le scratchpad. Un fichier créé par erreur se **déclare** au
  rapport ; il ne se supprime pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`, sous
  aucune forme.
- Aucune commande `gh` qui écrit (`issue create|edit|close|comment`, `pr …`) ; `gh issue view` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo` (aucune sous-commande), ni `npm`, ni `npx`, ni `make`, ni
  `latexmk`, ni `python3`, ni `node`, ni `sqlx`, ni `docker`, ni `mariadb`, ni `curl`, ni `claude`.
- Jamais `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `scripts/regen-test-schema.sh`,
  ni aucun script du dépôt.
- Jamais `rm`, `pkill`, `killall`.
- Jamais `~/kesh-test/` ; aucune donnée réelle dans le rapport.
- Commandes permises : `cat`, `sed -n`, `grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `git log`, `git show`,
  `git diff` (en lecture), `pdftotext … -` vers la sortie standard, `gh issue view`.
