# Prompt — validation P5 **ciblée** de la sous-fiche v016-2b (#816), une lentille, Sonnet

Tu fais une **passe ciblée** (`CLAUDE.md` § *La passe ciblée*) sur la sous-fiche `v016-2b-releve-manuel-ecran.md` de Kesh (comptabilité suisse,
Rust/Axum, SvelteKit, MariaDB 10.11) : la saisie manuelle d'un relevé bancaire (#816), découpée en `v016-2a` (backend)
et `v016-2b` (écran), release 0.17.0. **Une seule lentille**, braquée sur **le seul diff de la remédiation P4**. Modèle :
**Sonnet**. **Contexte frais**. Langue : **français**.

Worktree `/home/gcorbaz/devel/kesh-0162`, branche `story/v016-2-releve-manuel` ; code et manuels de `c5101b07` (la branche
ne touche que `_bmad-output/`).

## À lire

1. Le diff de la remédiation P4 : `git log --oneline -12`, puis
   `git show <commit « remédiation de la validation P4 des sous-fiches 2a et 2b »> -- _bmad-output/` — toutes les lignes
   ajoutées ou récrites de `v016-2b-releve-manuel-ecran.md`, de `v016-2a-releve-manuel-backend.md`, de la parente `v016-2-releve-manuel.md` et du registre
   `v016-2-choix-autonomes.md` (dont l'entrée **C-v016-2-19**).
2. Les rapports P4 qui l'ont motivé : `/home/gcorbaz/devel/kesh-gate-logs/v0162a-validate-p4.md` et
   `v0162b-validate-p4.md`, et l'entrée « Validation P4 » du Change Log de chaque sous-fiche.
3. Le code que le diff cite, ligne par ligne (`sed -n`, `grep -nF`).

## Axes (tous obligatoires)

1. **Le diff P4 de cette sous-fiche.** Chaque passage ajouté ou récrit : chaque `fichier:ligne` dit-il ce qu'on lui fait
   dire ? Chaque remède tient-il au code sans contredire un autre AC ? Chaque finding déclaré « appliqué » l'est-il
   entièrement (greper la **valeur** corrigée dans les trois fiches et le registre) ? Une entrée passée du Change Log
   a-t-elle été réécrite (interdit) ? Le « LOW non pris » est-il motivé ?
2. **La frontière entre 2a et 2b** — axe principal. Chaque élément partagé a-t-il **un seul** propriétaire, écrit
   des deux côtés en termes compatibles : la 2b ne touche-t-elle plus aucun repli de `errors.rs` ni
   `bank_accounts_e2e.rs:2485`, et garde-t-elle le Vitest `bank-accounts-page.test.ts:502`, cohérent avec le texte que la
   2a récrit ? `fiscal-years-checks-k2-help` et `bank-accounts-close-dialog-body` (numéros par catalogue,
   `closing-checks-labels.ts:48`) : justes ? La forme imposée à la table TypeScript (`manual-statement-reason-label.ts`,
   `MANUAL_STATEMENT_REASON_LABELS`, motif de clé) est-elle compatible avec le patron de la famille i18n
   (`MOTIFS_DYNAMIQUES`, `csv-line-error-label.ts`) et avec le test croisé et `ALL` de la 2a ? Le message générique du
   `400 VALIDATION_ERROR`, le `field` d'entête et la séquence de la raison sont-ils décidables et compatibles avec les
   ensembles fermés de la parente ? Le retrait d'`isPrimary` et de l'IBAN unique laisse-t-il l'AC-26 exécutable
   (`helpers/test-state.ts`, `wiredBankAccountViaApi`, preset `with-company`) ?
3. **Le registre** : C-v016-2-19 consigne-t-il exactement les trois décisions de la P3 (règle unique `400` et extracteur
   partagé ; épinglage Rust des `reason` et `field`, exhaustivité et test croisé ; statut de candidate de
   `README.md:235`), avec ses alternatives écartées ? Les bornes « C-v016-2-1 à C-v016-2-19 » des deux sous-fiches et
   les entrées P3 et P4 du Change Log de la parente sont-elles justes ?

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, à `/home/gcorbaz/devel/kesh-gate-logs/v0162b-validate-p5.md`. Il
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
