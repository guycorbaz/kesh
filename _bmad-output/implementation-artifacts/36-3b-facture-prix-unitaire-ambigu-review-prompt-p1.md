# Prompt — revue de code P1 de la Story 36-3b (#1171), trois lentilles, première passe complète

Tu relis la **Story 36-3b** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB 10.11), release **0.17.8**,
Epic 36, phase 1 : **le prix unitaire d'une ligne de facture client ne lit plus « 1.250 » comme 1,25 franc en silence** —
il se lit par `readJournalAmount` (grammaire du journal, Story 36-3, réemployée sans modification) : « 1.250 » est refusé
comme **ambigu**, « 12'34 » comme **illisible**, sous le champ (rangée sur les sept colonnes, bordure) et à l'envoi (bandeau
qui nomme la ligne) ; « 1'250 » et « 1,25 » sont acceptés et partent normalisés (`1250`, `1.25`) ; un prix enregistré au
millième se rouvre sur quatre décimales (`1.2340`, non `1.234`) et n'est pas refusé à tort ; la charge relit le prix après
l'attente des réglages. **Le serveur ne change pas** ; seuls les catalogues `crates/kesh-i18n/locales/*/messages.ftl` sont
touchés sous `crates/`. **Aucune migration.**

Worktree `/home/gcorbaz/devel/kesh-36-3b`, branche `story/36-3b-facture-prix-unitaire-ambigu`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-36-3b diff b837ee8de <DERNIER_CODE>` (`b837ee8de` = tête de la fiche validée, base de la
branche ; `<DERNIER_CODE>` est donné au § *Commits* ci-dessous). Langue du rapport : français. Modèle : **Sonnet ou Opus**
(première passe : jamais Haiku). Contexte frais.

## Commits

- `9a68c086d` — le correctif et ses tests (`unit-price.ts` et son test, `InvoiceForm.svelte` et son test, quatre
  catalogues, `i18n-keys.test.ts`) ;
- `de75ba63d` — l'E2E (`invoices.spec.ts`) et le manuel (`user-manual.tex`) : **dernier commit de code**,
  `<DERNIER_CODE>` = `de75ba63d` ;
- les commits suivants ne portent que la fiche, le registre des choix, ce prompt et le registre de sprint
  (`git diff --stat de75ba63d HEAD`).

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/36-3b-facture-prix-unitaire-ambigu.md` en entier (D1 à D7, AC1 à AC10, mutations M1 à
M13, annotations « Étape 0 » qui traitent V1-1 à V1-7, **Change Log** et **Dev Agent Record**) ; les registres
`e36-choix-autonomes-e360.md` (C-e360-11, C-e360-17) et `e36-choix-autonomes-36-3b.md` (C-36-3b-1 à C-36-3b-8) ; le rapport
de validation `/home/gcorbaz/devel/kesh-gate-logs/e36-3b/validation-p1.md`. Ne prends pas le Dev Agent Record pour argent
comptant : recoupe-le avec les journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e36-3b/` :
`etape0-inventaires.log`, `rouges.log`, `e2e-rouges.log`, `cible-*.log`, `check-*.log`, `e2e-cible-vert.log`,
`mut/resume.log` et `mut/M*.log` (script `mut/mut-36-3b.py`), `test-fast-ci-review.log`, `gate-ci-review.log`,
`front-*-review.log`. **Captures** (thème clair, AC10) sous `/home/gcorbaz/devel/kesh-gate-logs/e36-3b/captures/`
(`01-prix-1.250-ambigu-sous-le-champ.png`, `02-prix-1.250-ambigu-bandeau-apres-creer.png`, `03-prix-12-34-illisible.png`,
`04-prix-1-250-fois-2-total-2500.png`, `05-brouillon-rouvert-prix-1.2340.png`) — regarde-les si ton outil lit les images.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis tes trois rapports. **Aucun fichier temporaire**,
  nulle part ; **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni **`pkill`**, ni **`killall`**, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`,
  `bash -c`/`sh -c`, `jq` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`,
  `scripts/recette-montee.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` (dont
  `mut/mut-36-3b.py`, `e2e-36-3b.sh`, `captures-36-3b.cjs`) ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `sqlx`,
  `curl`, **`claude`**, `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk`
  (sans redirection ni `system()`), `cat`, `cat -A`, `od -c`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`,
  `cmp`, et, pour un PDF, la seule forme `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`.
- ⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport. Pour
  un classement de saisie, **déroule la regex et le prédicat à la main** contre le code lu ligne à ligne
  (`balance.ts:90` `JOURNAL_AMOUNT_RE`, `amount-ambiguity.ts:38-44`).
- **Tout chemin cité** se vérifie (`ls` ou `git cat-file -e <ref>:<chemin>`), la ligne lue citée.

## Les axes obligatoires, pour chaque lentille

1. **La lecture et la charge (D1, D3, C-36-3b-1, -2)** — `frontend/src/lib/features/invoices/unit-price.ts` :
   `unitPricePayload` rend-il `null` pour le vide (jamais `'0'`) ? `unitPriceProblem` laisse-t-il le vide sans message ? Une
   saisie claire part-elle **à la valeur que l'utilisateur voit** ? Une forme que le serveur refuse
   (`crates/kesh-api/src/routes/invoices.rs:506-520` : signe, plus de quatre décimales, plus de 10⁹) passe-t-elle `clear` sans
   que le serveur la refuse proprement ? `balance.ts` et `amount-ambiguity.ts` **inchangés**
   (`git diff b837ee8de <DERNIER_CODE> -- frontend/src/lib/features/journal-entries/ frontend/src/lib/shared/utils/` vide).
2. **La forme d'un prix rouvert (D2, C-36-3b-3)** — `unitPriceFieldValue` : aucune sortie ambiguë pour toute valeur
   `DECIMAL(19,4)` du serveur ? Déroule « 1.2340 », « 999.1230 », « 1000.1230 », « 0.1230 », « 0.0010 », « 12.3400 »,
   « 100.0000 », « 1234567.8900 ». Les trois sites (`initLines`, `onProductSelect`, `reloadFromServer`) l'appellent-ils
   tous, et la quantité garde-t-elle `readableAmount(…, 0)` ? Un autre site qui charge un prix dans un champ de facture
   client (`git grep -n "unitPrice" -- frontend/src ':!*.test.ts'`) ?
3. **L'écran (D3, D4, D5)** — `frontend/src/lib/components/invoices/InvoiceForm.svelte` : `validateClient` (ordre :
   description, puis prix, puis quantité ; message ambigu, illisible, vide) ; la rangée sous la ligne (`colspan` =
   `LINE_TABLE_COLUMNS` = 7, nombre réel de colonnes) ; la bordure ; les totaux par la charge ; la boucle de la charge après
   `await getInvoiceSettings()` et sa sortie anticipée (`submitting` remis à faux, aucune requête) ; *Entrée* passe-t-il par
   la même garde ? Un `$derived` ou un `{#each}` qui garderait un état périmé ? La rangée de message dérange-t-elle un
   sélecteur existant (`tbody tr`, E2E `invoices.spec.ts`, tests du composant) ?
4. **Textes (AC7)** — deux clés × quatre locales à côté d'`invoice-line-vat-rate-disabled-error` (jamais en fin de fichier) ;
   replis littéraux égaux au fr-CH ; fallbacks des deux clés de la 36-3 identiques à `JournalEntryForm.svelte` ; traductions
   qui disent la même règle ; `sitesTotal` 3040 → 3044 (`i18n-keys.test.ts`) ; `lint-i18n-ownership`.
5. **Le manuel contre le code (AC8)** — `docs/manual/fr/user-manual.tex`, item **Lignes de facture** du § *Création d'une
   facture* : chaque phrase est-elle vraie au code (forme admise, exposant, signe, `.5`, `5.`, `1.250` refusé, message qui
   nomme la ligne, `1.2340` rouvert, quantité non concernée) ? **Le PDF n'est pas régénéré dans la branche** (fiche AC8, à
   l'intégration) : vérifie-le (`git diff --stat b837ee8de <DERNIER_CODE> -- docs/`) et dis la sonde à jouer à
   l'intégration : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'est refusé comme ambigu'`
   (attendu ≥ 1 après régénération ; 0 aujourd'hui).
6. **Les témoins et les mutations** — chaque AC a-t-il un test qui peut rougir ? M1 à M14 et M9b jouées et tuées
   (`mut/resume.log`) ; l'aide `borde()` du test lit-elle `classList` (la classe de base d'`Input` contient
   `aria-invalid:border-destructive`, qu'un `toContain` sur `className` confondrait) ? Les tests d'AC5 retapent-ils vraiment
   le prix **pendant** l'attente ? Aucun attendu de test existant retouché (`git diff b837ee8de <DERNIER_CODE> -- '*.test.ts'
   '*.spec.ts'`, lignes `-`) ? Propose des mutations plausibles non jouées.
7. **Le périmètre (AC9)** — `git diff --stat b837ee8de <DERNIER_CODE>` ne nomme ni `crates/` hors `kesh-i18n/locales/`, ni
   `journal-entries/`, ni `shared/utils/amount-ambiguity.ts`, ni `supplier-invoices/`, ni `products/`. Les sites hors
   périmètre de l'inventaire (fiche, § *Inventaire par le symptôme*) gardent leur issue.

## Lentille B — Blind (le diff seul, sans la fiche)

Relis ligne à ligne `unit-price.ts`, le diff d'`InvoiceForm.svelte`, les quatre catalogues, les tests et l'E2E. Cherche :
une valeur envoyée différente de la valeur lue ; un prix vide ou `null` qui partirait ; un refus à tort ; un test qui passe
**à vide**.

## Lentille E — Edge (chaque branche, chaque borne)

Déroule au code, pour la saisie : `0.250`, `000.125`, `1.000`, `1000.000`, `1'250`, `1’250,5`, `1'250.12345`, `12'50`,
`0'250`, `999'999'999`, `1'000'000'000.0001`, `1 250` (espace, U+00A0, U+202F), `1‘250` (U+2018), `.5`, `5.`, `-0`, `+5`,
`1e3`, `１２３` (pleine chasse), une saisie suivie d'un retour à la ligne, un blanc de tête ; pour le serveur rouvert : les
valeurs de l'axe 2 ; pour l'envoi : un prix retapé, vidé, puis rétabli pendant l'attente ; deux lignes dont la seconde seule
est ambiguë (le message nomme-t-il « Ligne 2 » ?). Chaque cas : issue au code, issue attendue (fiche), test qui le couvre
(ou absence, prouvée par `grep -nF`).

## Lentille A — Acceptance (la fiche contre le livré)

Chaque critère AC1 à AC10 : livré, testé, au lieu prescrit (y compris les déplacements de l'étape 0 : `unit-price.ts` au lieu
d'`invoice-helpers.ts`) ? Les rouges de départ (`rouges.log`) ; `e2e-rouges.log` ; les verts au départ dits tels ; les
mutations ; les gates au dernier commit de code (chiffres du Dev Agent Record, chacun retrouvé au journal) ; les écarts
déclarés. Le Dev Agent Record n'affirme-t-il que ce qui a tourné (E2E complet et `gate-ci` de clôture **non lancés** : ils
tournent à la clôture) ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/e36-3b/review-p1-B.md`, `…/review-p1-E.md`,
`…/review-p1-A.md`. Chaque rapport : findings numérotés (`B1`, `E1`, `A1`…) avec sévérité (CRITICAL / HIGH / MEDIUM /
LOW), lieu (`fichier:ligne`), ligne lue copiée, rejeu ou preuve, remède proposé ; **la liste des axes exercés ET non
exercés** ; le décompte final **recompté depuis la liste**.
