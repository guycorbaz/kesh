# Prompt — revue de code P3 de la Story v016-2b (saisir un relevé à la main, l'écran) — passe ciblée, Sonnet

Tu relis **la seule remédiation de la revue de code P2** de la Story v016-2b de Kesh (comptabilité suisse, SvelteKit 5,
Rust, MariaDB ; #816). Passe **ciblée**, **une lentille**, modèle **Sonnet**, contexte frais : ne t'appuie sur aucune passe
antérieure. Langue du rapport : français.

Worktree `/home/gcorbaz/devel/kesh-0163`, branche `story/v016-2b-releve-manuel-ecran`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-0163 diff c3f9d6e2...HEAD` (`c3f9d6e2` = la fin de la remédiation P1, relue en P2).
Le rapport P2 : `/home/gcorbaz/devel/kesh-gate-logs/v0162b-review-p2.md` ; ce qui a été appliqué ou non : l'entrée
« Revue de code P2 » du Change Log de `_bmad-output/implementation-artifacts/v016-2b-releve-manuel-ecran.md`.

⛔ Ne lis rien sous `~/kesh-test/` (données réelles).

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport ; aucun fichier temporaire ; **aucune
  redirection** (`>`, `>>`, `tee`, `2>`, `&>`). Un fichier créé par erreur se déclare, il ne se supprime pas.
- **Aucun `rm`**, ni `pkill`, ni `killall` ; **aucun `git checkout`/`git switch`/`git restore`**, aucun `git` qui écrit.
- **Rien qui compile ou exécute** : ni **`python3`**, ni `node`/**`npm`**/`npx`, ni **`cargo`**, ni **`make`**, ni
  **`docker`**, ni `mariadb`, ni **`curl`**, ni **`claude`**, ni **`sed -i`**, ni **`scripts/prepare-release.sh`**, ni
  aucun script du dépôt ou de `kesh-gate-logs`. **Aucune commande `gh`**, aucun accès réseau.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `grep`, `sed -n`, `awk` (sans redirection), `cat`, `ls`,
  `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`. Toute affirmation de présence ou d'absence s'appuie sur une
  sortie `grep -nF` (ou `sed -n`) copiée.

## Les axes de la lentille

1. **L'avis du refus de création** (`ManualStatementForm.svelte`, `save`, `error.serverChanged`, bloc
   `bank-import-manual-preview-outdated-server`) : s'affiche-t-il pour les trois codes de `NEEDS_NEW_PREVIEW`, et pour
   eux seuls ? Disparaît-il quand le relevé change (le refus tombe avec sa clé), et à la vérification suivante ?
   L'avis « le relevé a changé » (`preview-stale`) ne s'affiche-t-il plus dans ce cas ? Un refus de création qui ne
   demande pas de case (`409 RECONCILIATION_ACCOUNT_LOCKED`, `BANK_ACCOUNT_CLOSED`) garde-t-il l'aperçu ? Les trois
   Vitest « M33 » prouvent-ils ce qu'ils nomment, et la mutation M33 (journal
   `/home/gcorbaz/devel/kesh-gate-logs/v0162b/mutations/M33.log`) est-elle tuée par eux ?
2. **La clé neuve** `bank-import-manual-preview-outdated-server` : repli égal au fr-CH, quatre catalogues traduits et
   cohérents, `REPLIS_A_SITE_UNIQUE` (94, recompté), `sitesTotal` (2714, recompté par la commande du commentaire). Son
   texte dit-il vrai (rien n'est enregistré ; il faut vérifier puis confirmer) ?
3. **L'en-tête du composant et le Change Log** (P2) : disent-ils exactement quand l'aperçu se périme (une ligne vide en
   fin de tableau ne périme rien, et c'est voulu) ?
4. **La garde de la page** (P4) : le test vérifie-t-il la redirection exacte (302 vers `/`), et peut-il passer à vide
   (aucune exception levée → `leve === null`) ?
5. **Le témoin renommé** (P5) `TemoinFormulaire.test.helper.svelte` : exclu des gardes qui écartent `.test.`
   (`boutons-rouges-inventory.test.ts`, `jetons-non-definis.test.ts`, moissonneur i18n) ? Une autre garde le prend-elle
   pour un fichier de test exécutable, ou le perd-elle ? L'import du test suit-il le nouveau nom ?
6. **G24** (P7, `crates/kesh-api/tests/textes_coherents.rs`) : l'ancre `(?m)^<ouverture>[:= ]` écarte-t-elle un nom
   prolongé et une mention en commentaire, et trouve-t-elle la déclaration réelle
   (`manual-statement-reason-label.ts:21`) ? Le témoin neuf couvre-t-il le nom prolongé placé **avant** la vraie table ?

## Rapport

Un seul fichier : `/home/gcorbaz/devel/kesh-gate-logs/v0162b-review-p3.md`, par l'outil d'écriture, jamais par une
redirection. Contenu :
1. **Les findings** : identifiant (`P1`, `P2`…), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier et ligne, la
   preuve copiée, la correction proposée, et s'il est **né de la remédiation** ou antérieur.
2. **Les axes réellement exercés, et ceux qui ne l'ont pas été** — sans cette liste, un « 0 finding » ne compte pas.
3. **Le décompte** `CRITICAL / HIGH / MEDIUM / LOW`, recompté depuis ta liste.
4. Les fichiers créés par erreur, et toute commande lancée hors de la liste permise.
