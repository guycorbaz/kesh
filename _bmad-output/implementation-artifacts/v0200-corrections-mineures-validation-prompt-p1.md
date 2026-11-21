# Prompt — validation P1 de la fiche `v0200-corrections-mineures` (#1348, #1336, #1342, #1349, #1343), passe complète

Tu valides, **en lecture seule**, la fiche de story `v0200-corrections-mineures` de Kesh (comptabilité suisse ; Rust/Axum, MariaDB,
SvelteKit 5 runes), release **0.20.0** (coupure le jeudi 2026-11-26 au soir). Kesh tient les livres réels du Project Lead (Guy), qui
a **déjà émis de vraies factures remises sur papier** : rien de ce qui existe (facture, avoir, lignes, totaux, TVA, écritures,
numéros, PDF) ne doit changer. **Aucune migration** n'est permise à cette story.

- **Modèle : Opus. Contexte frais. Passe complète** (première passe : jamais ciblée, jamais Haiku). Langue du rapport : français.
- Worktree : `/home/gcorbaz/devel/kesh-v0200-cm`, branche `story/v0200-corrections-mineures`, partie de
  `epic/48-corrections-0-20-0` = `a051e17e5`. Fiche :
  `/home/gcorbaz/devel/kesh-v0200-cm/_bmad-output/implementation-artifacts/v0200-corrections-mineures.md`.
- Lis d'abord : le `CLAUDE.md` du worktree (Qualité du code, Recette visuelle, Review Iteration Rule, Inventorier les sites NON
  RÉSOLUS, Propagation post-patch, Priorités des défauts, Issue Tracking Rule, Règle de splitting préventif, Documentation à
  synchroniser) ; `/home/gcorbaz/devel/kesh-gate-logs/prompts/e0200-commun.md` ; la fiche de la story sœur
  `_bmad-output/implementation-artifacts/48-15-troncatures-silencieuses.md` (motif réutilisé, règle de décompte des modules) ; les
  cinq issues (`gh issue view <N> --json number,title,body,comments`, lecture seule) : #1348, #1336, #1342, #1349, #1343 ; et #1375
  (déclarée hors périmètre).

## Interdits

Tu n'écris **aucun fichier** hors de ton rapport. En particulier, tu ne lances **ni** `cargo` (build, test, nextest, clippy, fmt),
**ni** `npm` / `npx` (check, test, build, lint), **ni** `make`, **ni** aucun `scripts/*` (dont `scripts/prepare-release.sh`,
`scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/mem-guard.sh`, `scripts/recette-montee.sh`, `scripts/install-hooks.sh`),
**ni** `docker` / `docker compose`, **ni** `mariadb` / `sqlx`, **ni** `git commit`, `git checkout`, `git stash`, `git restore`,
`git reset`, `git push`, `git worktree`, **ni** `gh` en écriture (`issue create|edit|comment|close`, `pr …`), **ni** `sed -i`, `tee`,
`>` ou `>>` vers un fichier du dépôt. Rien sous `~/kesh/`. Permis : `git log`, `git show`, `git diff`, `git grep`, `grep`, `sed -n`,
`cat`, `pdftotext … -` (vers la sortie standard), `gh … view|list`, et un `python3` qui ne fait que **lire**.

Rapport : **`/home/gcorbaz/devel/kesh-gate-logs/v0200cm/validation-p1.md`** (le seul fichier que tu écris).

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

1. **Causes au code.** Chaque cause citée (`chemin:ligne`) est relue à `a051e17e5` ; une ligne fausse, une prémisse démentie, une
   cause plus profonde manquée est un finding. En particulier : `errors.rs:76-81` et `loader.rs:110-124` (la clé brute rendue) ;
   `address_input.rs:46-51`, `:160-169` et le préremplissage « CH » de l'écran ; les branches de `formValidation`
   (`contacts/+page.svelte:476-491`) et le bouton (`:1162-1165`) ; `onboarding.rs:526-531`, `contacts.rs:379`, `:397`,
   `contact_persons.rs:86-92`.
2. **Ce que la 48-15 a déjà réglé.** Vérifie au code et au diff de la 48-15 (`git log --oneline af4e0e313^1..af4e0e313^2`,
   `git diff af4e0e313^1 af4e0e313 -- 'frontend/src/routes/(app)/contacts/+page.svelte'`) ce que la fiche dit réglé (onze `maxlength`,
   délai, motifs sous le champ) et ce qu'elle dit restant (`:987`, `:992`). Rien ne doit être refait, rien d'oublié.
3. **Inventaires du symptôme, ensembles clos.** Refais toi-même, en lecture seule : (a) l'extraction des clés appelées par `t(` /
   `t_args(` avec un littéral dans `crates/kesh-api/src/**/*.rs` (modules de test exclus) contre `fr-CH/messages.ftl` — le compte
   (vingt clés absentes) et la liste sont-ils justes ? (b) les sites à clé **calculée**, par fichier (la fiche dit 27 : `errors.rs` 23,
   `opening_balances.rs` 1, `invoice_pdf_service.rs` 2, `companies.rs` 1) ; (c) les contrôles « prénom / nom / raison sociale
   renseignés » jugés par `trim` seul dans les routes qui écrivent un nom (la fiche en compte trois à corriger, un résolu). Un site
   manqué, ou une assertion vraie par construction (les deux côtés sortant du même extracteur), est un finding.
4. **Décisions et périmètre.** D1 b (dix-neuf clés de plus que #1348) et D5 (contacts et interlocuteurs en plus de l'assistant) sont-ils
   justifiés par le `CLAUDE.md` (propagation, une issue par problème) ou un élargissement à renvoyer en issues ? D3 (« CH » seul reste
   « pas d'adresse ») tient-il au code (préremplissage, `normalized_country`) ? D4 (le motif dérivé de `formValidation`, exception
   « déjà affiché ») ferme-t-il la classe de #1349 sans afficher deux fois un même texte, y compris après une soumission refusée ?
   D6 (#1375 à la 48-12b) est-il exact (la 48-12b réécrit-elle `unknown-outcome.ts` ? lis sa fiche si elle existe, `git log --all`).
5. **Cohérence écran / serveur.** Chaque borne et chaque refus de l'écran comptent comme le serveur (caractères, rognage, invisibles) ;
   aucun refus muet ; un refus serveur s'affiche et la saisie reste. Le panneau du fournisseur neuf, non modifié, affiche-t-il bien le
   refus neuf du pays seul ?
6. **Données existantes et factures réelles.** Aucun contact, aucune société, aucun interlocuteur existant ne doit être refusé sauf la
   limite déclarée à l'AC12 b ; aucune facture ni aucun PDF ne change (AC12 a) ; aucune migration n'est requise (vérifie).
7. **Critères et tests.** Chaque AC est testable et rougit pour la bonne raison ; mutations M1 à M16 uniques, chacune tuée par un test
   nommé ; sondes positives avant chaque assertion d'absence ; filets déclarés ; aucun test vrai par construction ; E2E pour chaque
   comportement visible (et la raison donnée pour #1343 et #1348 tient-elle ?) ; sélecteurs `data-testid` (garde KF-043).
8. **i18n.** Clés aux quatre locales (fr, de, it, en), `parity_between_locales`, `dette-parite-connue.txt`, `les_replis_rust_suivent_le_catalogue`,
   `lint-i18n-ownership`, `sitesTotal` de `i18n-keys.test.ts` recompté ; messages serveur restés en français (#710) déclarés.
9. **Manuel, PDF compris.** Vérifie `docs/manual/fr/user-manual.tex` (lignes 219, 1350, 1353, 1354 et voisines) **et** le PDF publié,
   aplati avant de greper : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o '.\{0,200\}<motif>.\{0,200\}'`
   (motifs : « Email », « Téléphone », « Adresse structurée », « grisé », « trait d'union conditionnel », « pays »). Ce que le manuel dit
   aujourd'hui de ces comportements contredit-il le code, avant ou après la story ? Les manuels DE/IT/EN décrivent-ils ces items ?
   L'AC14 suffit-elle ?
10. **Règle de splitting.** Recompte les modules au grain du domaine (règle de la 48-15) depuis la liste des fichiers que la fiche
    touchera ; au-delà de cinq, finding.
11. **Notes de version et documentation.** Une puce par issue, sans nommer d'epic, numéro de release seulement ; la phrase pour
    « Passer à la 0.20.0 » dit les changements de l'API ; aucun mot-clé de fermeture (`closes`, `fixes`, `resolves` et variantes)
    dans la fiche ni dans les messages de commit (`git log origin/epic/48-corrections-0-20-0..HEAD --format=%B`).
12. **Fichiers communs et coordination.** Conflits possibles avec les autres stories de la release (48-11, 48-12a/b, 48-14, Epic 49,
    v0200-menu-sous-groupes) et la PR #1327 du contributeur ; `sprint-status.yaml` (clé ajoutée au bloc de l'Epic 48).

## Rapport

Pour chaque finding : identifiant `V1-n`, sévérité (CRITICAL, HIGH, MEDIUM, LOW), axe, constat **avec la ligne lue citée**
(`chemin:ligne` et l'extrait), conséquence, remède proposé. Pour tout CRITICAL ou HIGH qui affirme l'absence d'un code ou la présence
d'un anti-pattern, la commande `grep -nF` qui le prouve et sa sortie.
En fin de rapport : décompte **CRITICAL / HIGH / MEDIUM / LOW**, recompté depuis tes findings ; la liste des **axes exercés** et des
**axes non exercés** (avec la raison) ; les commandes lancées. Un « 0 finding » sans cette liste ne compte pas comme passe.
