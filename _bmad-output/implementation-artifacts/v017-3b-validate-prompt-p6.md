# Prompt — validation P6 (ciblée) de la fiche v017-3b (#687 : une facture fournisseur au centime), une lentille, Opus

Tu valides la **remédiation de la passe P5** de la fiche de la Story v017-3b de Kesh (comptabilité suisse, Rust/Axum,
SvelteKit, MariaDB 10.11). Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*) : **une seule lentille**, braquée sur le
**seul diff** du commit `docs(v017-3b) : remédiation de la validation P5 (refs #687)`. Modèle : **Opus**. **Contexte
frais** : ne t'appuie sur aucun rapport antérieur pour juger, ni sur ce que la fiche dit d'elle-même. Langue du rapport :
**français**.

Worktree `/home/gcorbaz/devel/kesh-0173`, branche `story/v017-3-rapprochement-attente`, partie de `c5101b07`.
Historique : P1 0/0/11/14 ; P2 0/0/11/17 (simplification C-v017-3-24) ; P3 0/0/1/18 (C-v017-3-26) ; P4 0/0/3/7
(C-v017-3-27 : décimales exactes sur la seule fiche de la facture ; la liste garde son format, #831) ; P5 (Sonnet,
ciblée) 0/0/1/5.

## À lire

1. Le diff ciblé : `git log --oneline c5101b07..HEAD`, puis `git show <commit de la remédiation P5>` (fiche
   `v017-3b-facture-fournisseur-au-centime.md`, registre `v017-3-choix-autonomes.md`).
2. La fiche entière et les entrées C-v017-3-18, -24, -26, -27 du registre **après** le diff.
3. Le rapport P5, **après** ton examen : `/home/gcorbaz/devel/kesh-gate-logs/v017-3b-validate-p5.md`.

## La lentille

### Axe obligatoire — la cohérence entre le registre et la fiche sur l'affichage

Établis, en lisant **ensemble** la fiche et le registre, ce que la story affiche et où : l'avertissement (états où il
s'affiche, texte, format du montant), la cellule TTC, les cellules de lignes, le total du sélecteur de document, le
montant du QR, la liste. Pour chacun, la fiche (D5, D8, D10, D11, AC-10, AC-14, M7, M11, M14, M16, M17, L4, issues) et le
registre (C-v017-3-18, -24, -26, -27 et leurs amendements) disent-ils **la même chose**, compte tenu des marques
« remplacé par » ? Grepe la **valeur** `formatExactAmount`, `liste`, `+page.svelte:93`, `AttachDocumentPicker`,
`annulée`, `#831` dans les deux fichiers ; toute phrase vivante (non marquée remplacée, non historique datée) qui
contredit C-v017-3-27 ou C-v017-3-26 est un finding.

### Ce que la remédiation a pu introduire

- Chaque prémisse introduite, **au code** : `crates/kesh-api/tests/onboarding_audit_e2e.rs:903` (un `sqlx::raw_sql`
  y est-il employé ?) ; la version de `sqlx` du `Cargo.lock` ; `crates/kesh-api/src/routes/reconciliation.rs:1252-1258` ;
  `KESH_INBOX_DIR` et le dépôt d'un document « à traiter » ; #831 (`gh issue view 831`).
- M16 : le texte d'origine est-il restauré **mot pour mot** (`git show <commit P3>:…` pour le comparer) et l'annotation
  ajoutée sans le modifier ?
- L'ordre de la recette de démonstration (créer, jouer le fichier, payer, déposer un document) produit-il l'état exact
  d'une base 0.16.x pour chaque capture de l'AC-14 ?
- Les amendements du registre sont-ils placés **sous** l'entrée qu'ils amendent ?

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, à `/home/gcorbaz/devel/kesh-gate-logs/v017-3b-validate-p6.md`. Il
contient :

1. **Les findings**, numérotés `C1`, `C2`…, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), le lieu
   (`fichier:ligne`), **la ligne lue citée** (sortie `grep -nF` ou extrait copié) et le remède proposé. Une affirmation de
   présence ou d'absence sans sortie `grep -nF` copiée ne compte pas.
2. **Les axes exercés** et **les axes non exercés**, un par un, avec ce que tu as lu ou cherché. Un « 0 finding » sans
   cette liste ne compte pas comme passe.
3. Un décompte final par sévérité, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture**, hors de ton seul rapport : ni `Edit`, ni `Write` sur un autre fichier, ni redirection (`>`, `>>`,
  `tee`) — pas même vers un fichier « jetable ». Un fichier créé par erreur se **déclare** au rapport ; il ne se supprime
  pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`, sous
  aucune forme.
- Aucune commande `gh` qui écrit ; `gh issue view` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo`, ni `npm`, ni `make`, ni `latexmk`, ni `python3`, ni `node`,
  ni `sqlx`, ni `docker`, ni `curl`, ni `claude`.
- Jamais `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt.
- Jamais `rm`, `pkill`, `killall`.
- Jamais `~/kesh-test/` ; aucune donnée réelle dans le rapport.
- Rien dans `/home/gcorbaz/devel/kesh-0162` ni dans `/home/gcorbaz/devel/kesh` : lecture seule.
- Commandes permises : `cat`, `sed -n`, `grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `git log`, `git show`,
  `git diff` (en lecture), `pdftotext … -` vers la sortie standard, `gh issue view`.
