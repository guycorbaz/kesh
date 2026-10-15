# Prompt — validation P2 de la spec, Story 29-9 (trois petits correctifs : #649, #655, #646)

*Versionné le 2026-10-14. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 était Sonnet ; P2 : **Opus**
pour les deux lentilles.*

Dépôt `/home/gcorbaz/devel/kesh-29-9`, branche `story/29-9` (base `fcd2ae75`, kickoff de l'Epic 29).
Fiche : `_bmad-output/implementation-artifacts/29-9-trois-petits-correctifs.md`.
Registre : `_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-9.md` (C-29-9-1 à 16).
Commits : fiche `3176e1be`, prompt P1 `b33a874c`, **remédiation P1 `5f823dc6`** — le diff à relire en premier :
`git show 5f823dc6` (ou `git diff b33a874c 5f823dc6`, un seul diff aplati).
Rapports de la passe précédente : `/home/gcorbaz/devel/kesh-gate-logs/29-9-validate-p1-R.md` et `…-p1-F.md`
(R : 0/0/4/5 ; F : 0/0/5/6) ; Change Log de la fiche, entrée « Validation P1 remédiée ».
Epic : `_bmad-output/planning-artifacts/epic-29-preparer-cloture.md` § *29-9*, § *Ordre*, § *Releases*.
Issues (l'objet) : `gh api repos/guycorbaz/kesh/issues/649`, `…/655`, `…/646`.
Fiches **non mergées** qui partagent `ReconciliationProposals.svelte` et le manuel, en lecture seule :
`/home/gcorbaz/devel/kesh-29-3/_bmad-output/implementation-artifacts/29-3-meilleure-proposition.md`,
`/home/gcorbaz/devel/kesh-29-6/_bmad-output/implementation-artifacts/29-6-trop-percu-centimes.md`,
`/home/gcorbaz/devel/kesh-29-8/_bmad-output/implementation-artifacts/29-8-transaction-reglee-ailleurs.md`.
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — passe ciblée, manuel nommé, axes déclarés, sites
NON RÉSOLUS, propagation post-patch, recompte des comptes rendus, « recompter n'est pas réécrire l'historique » ;
§ Règle de splitting préventif ; § Issue Tracking Rule) ; `docs/agents-orchestration.md` ; rétrospective
`epic-28-retro-2026-10-14.md` § 4 et § 7.

## Axe 0 — OBLIGATOIRE, en premier : relire la remédiation P1

*La sévérité se déplace vers ce qu'on vient d'écrire* (CLAUDE.md, § passe ciblée). Pour **chacun** des 17 défauts
traités (R1-R9, F1-F11, avec les paires R1=F1, R2=F2, R3=F3), dis : (a) le correctif est-il **présent** dans la
fiche (`grep -nF` copié) ; (b) est-il **juste** au code de `fcd2ae75` ; (c) a-t-il **introduit** un défaut neuf —
contradiction avec une autre section de la fiche ou du registre, test qui ne rougirait pas, mutation intuable,
prescription infaisable. Points qui demandent un regard propre :

1. **AC6 réécrit (F4, C-29-9-13)** — `const rawId = $derived(page.params.id)` : est-il vrai, au source de Svelte
   5.55.1 et de `@sveltejs/kit` 2.55.0 du dépôt (`frontend/node_modules`), que (i) `page.params` est réassigné à chaque
   navigation dans la route, (ii) un `$derived` primitif recalculé à la même valeur ne relance pas l'effet qui le lit,
   (iii) dériver le nombre relancerait l'effet sur `NaN` ? `isLoading = true` est-il **établi** sans effet sur le
   formulaire à id égal, ou seulement affirmé ? Le `++loadSeq` avant la branche « ID invalide » est-il cohérent avec
   la garde des `.then`/`.catch` (V16) ? Y a-t-il une boucle d'effet possible (`effect_update_depth_exceeded`) ?
2. **Fixture réactive** (`class FakePage { params = $state.raw(...) }`, servie par un `vi.mock` asynchrone qui
   importe un `.svelte.ts`) : faisable avec `frontend/vite.config.ts` ? `beforeEach` qui la remet à `'5'` — et le
   nettoyage du rendu précédent ? V13-V16 rougissent-ils **au sol** pour la bonne raison, et chacun tue-t-il sa
   mutation (M13, M15, M16, M17) ? M15 n'est-elle tuée que grâce à la réassignation, comme l'écrit la fiche ?
3. **Mock `importOriginal` (R1/F1)** : `importOriginal` de `./reconciliation.api` charge le vrai `api-client` —
   effet de bord au chargement, dépendance que le test ne mocke pas ? Le repli `reconciliation.limits.ts` est-il
   cohérent avec C-29-9-4 (constantes « d'un seul endroit ») ?
4. **C-29-9-14 (`loading` non gardé)** : la preuve « les déclencheurs ne sont rendus que dans `{:else if detail}` »
   est-elle juste (`[id]/+page.svelte`, toutes les branches, `CancelReconciliationDialog` hors du `{#if}`) ?
5. **C-29-9-15 (saut de défilement déclaré)** et **C-29-9-16 (commentaire `kesh-db` laissé)** : les raisons
   tiennent-elles au code ? Le commentaire `kesh-db` est-il **vrai** après la story, ou faux et donc un site non
   résolu ?
6. **Second test Rust à exactement 500 (F8, M18)** : `seed_bank_transactions` supporte-t-il 500 lignes ; M18 est-elle
   formulée sur la ligne réelle (`routes/reconciliation.rs`, `has_more`) ?
7. **E2E (F11)** : la contrainte de date retirée l'est-elle partout (fiche **et** registre) ? L'unicité des entrées
   et le contrôle « 501 créées » sont-ils vérifiables par la réponse de l'import (forme de la réponse au code) ?
8. **Annotations datées** (C-29-9-4, 7, 8, 12 du registre ; prémisse de la fiche) : corrigent-elles sans réécrire
   l'historique, et ne laissent-elles aucune phrase contradictoire vivante ?

## Axes de la passe complète (communs aux deux lentilles)

1. **Prémisses** (§ *Le constat, au sol*, § *Les prémisses des issues*) revérifiées au code de `fcd2ae75`.
2. **#649, conception** (AC1-AC4, C-29-9-2 à 6, 15, 16) : palier unique, effet qui suit `limit`, double requête,
   avis au plafond sans le rejet, sélection vidée, reflet des constantes et tests Rust à la frontière.
3. **#655, conception** (AC5, AC6, C-29-9-7, 8, 13, 14) : garde de séquence sur toutes les écritures qui ont un effet
   observable ; effet du profil borné à l'identifiant.
4. **#646, conception** (AC7, C-29-9-10) : `window.confirm` dans un dialogue bits-ui ; vérité du texte de
   confirmation.
5. **Manuel contre code (obligatoire)** : `docs/manual/fr/user-manual.tex` l. 869-874 et 2043-2060, et ce que l'AC8
   prescrit (dont le **delta** à `:2057`) ; contrôle aussi le **PDF**, aplati :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`. Même exigence pour
   `docs/api-external.md:594`, les clés i18n (quatre locales), `reconciliation-unreject-success`, et les gardes i18n
   que l'AC8 nomme désormais (`i18n-entrees-a-variables.test.ts`, `i18n-repli-divergent-actif.test.ts`,
   `lint-i18n-ownership.js`). Un texte qui promettrait ce que le code ne tiendrait pas est au moins **MEDIUM**.
6. **Sites non résolus et propagation** : refais les `grep` du § *Propagation* (élargis par la P1) et élargis-les
   encore — tout texte qui affirme la limite de 100 sans suite, l'absence d'interface pour `hasMore`, la suppression
   sans confirmation, ou « l'effet ne se rejoue que si l'id change ». Un site oublié est au moins **MEDIUM**.
7. **Zones partagées** : le tableau complété par la P1 contre les trois fiches voisines (numéros de ligne relus).
8. **Découpage et périmètre** : dérogation C-29-9-1 ; chaque issue entièrement réglée ou exclue avec issue demandée.
9. **Recomptes** depuis la source : AC (10), T0-T9 (10), V1-V16 (16), M1-M18 (18), choix (16), clés neuves (4) et
   réécrite (1), `sitesTotal` au sol (2107) ; et le bilan de l'entrée « Validation P1 remédiée » (20 findings, 17
   distincts, 6 MEDIUM, 11 LOW) contre les deux rapports P1.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : axe 0 points 3-8, axes 1, 5-9 en priorité ;
  `.claude/skills/bmad-create-story/checklist.md` ; chaque « Attendu » des trois issues couvert par un AC testable,
  chaque AC par une tâche et par un test **qui rougirait** au sol ; chaque fait, chemin et numéro de ligne revérifié
  (`sed -n`, `grep -nF`).
- **F — Faisabilité et exactitude de la conception (adversaire)** : axe 0 points 1-6, axes 2-4 en profondeur ;
  tuabilité de **chaque** mutation M1-M18 par le test nommé ; faisabilité des fixtures (`.svelte.ts`, CAMT de 501) ;
  ce qui manque encore (un état d'écran, une écriture d'état, une course) sans test.

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée ; pour chacun, dis s'il vient **de la remédiation
P1** ou de la conception d'origine. Toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** (axe 0 point par point compris) — un « 0 finding » sans elle ne compte
pas comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/29-9-validate-p2-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/29-9-validate-p2-F.md` (lentille F) ; dernier message : le chemin du rapport, le
bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`),
`make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout` **sous aucune forme** (`--detach` compris)/`switch`/
`restore`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`/`worktree`, `cargo`, `npm`, `npx`, `node`, `vitest`,
`playwright`, **tout interpréteur** (`python3`, `python`, `perl`, `ruby`, `bash`/`sh` sur un script, `awk` qui
écrit), `docker`, **`mariadb`/`mysql`** et toute requête SQL, `gh` en écriture. Aucune donnée réelle de Guy ne se lit
ni ne se cite. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`/`blame`, `gh api` en lecture,
`pdftotext` vers la sortie standard.
