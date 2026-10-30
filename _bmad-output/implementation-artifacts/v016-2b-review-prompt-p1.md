# Prompt — revue de code P1 de la Story v016-2b (saisir un relevé bancaire à la main, l'écran), trois lentilles, Sonnet

Tu relis le développement de la **Story v016-2b** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5, MariaDB 10.11) :
*saisir un relevé bancaire à la main — l'écran* (#816). Ce qu'elle livre : la page `/bank-import/manual` (garde Comptable+,
`+page.ts`) et le lien « Saisir un relevé à la main » de `/bank-import` ; le formulaire
`frontend/src/lib/features/bank-import/ManualStatementForm.svelte` (aperçu puis enregistrement, écart en continu,
Débit/Crédit, raison unique d'un « Enregistrer » inactif, instantané de l'aperçu, table des rangs) ; la logique pure
`manual-statement.ts` ; `isValidAmount` **généralisée** (`journal-entries/balance.ts`) ; la table des `reason`
`manual-statement-reason-label.ts` et son test Rust croisé **G24** (`crates/kesh-api/tests/textes_coherents.rs`) ;
l'affichage d'un import `MANUAL` (`bank-import-display.ts`, `BankImportList.svelte`, `bank-import/[id]/+page.svelte`,
`DeleteBankImportAction.svelte`) ; le détail entièrement en clés ; les en-têtes de liste, la boîte de clôture d'un compte
et l'aide K2 récrits ; les clés aux **quatre** catalogues ; les gardes i18n recalées ; l'E2E
`frontend/tests/e2e/bank-manual-statement.spec.ts` (et `createUser` extraite vers `helpers/api-fixtures.ts`) ; le manuel
utilisateur FR (`.tex` **et** PDF : § *Saisir un relevé à la main*, phrases récrites, FAQ), `admin-manual.tex:82` (et PDF),
le CHANGELOG (*Ajouté*), le README (*Fonctionnalités*). Le backend (routes, codes, migration) est celui de la v016-2a :
il n'est pas l'objet de cette revue, sauf là où l'écran en dépend.

Worktree `/home/gcorbaz/devel/kesh-0163`, branche `story/v016-2b-releve-manuel-ecran`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-0163 diff 285d2fde...HEAD` (`285d2fde` = tête de la v016-2a développée). Langue du
rapport : français. Modèle : **Sonnet**. Passe **complète**. Contexte frais : ne t'appuie sur aucune passe antérieure.

⚠️ **Les propriétés centrales.**
1. **Un écart non nul n'enregistre jamais** ; « Enregistrer » ne s'active que sur un aperçu **réussi sur l'état courant**
   du relevé, cases exigées cochées ; la raison dite est la première de la séquence de l'AC-21.
2. **Le rang envoyé n'est jamais affiché tel quel** : toute ligne désignée par le serveur passe par la table des rangs de
   l'instantané envoyé (lignes vides retirées).
3. **Une réponse tardive ne compte pas**, une modification périme l'aperçu et décoche les cases, cocher ne périme pas.
4. **Les `reason` de l'écran égalent l'ensemble fermé du serveur** (G24, dans les deux sens).
5. **Aucun texte en dur** ; replis = valeur `fr-CH` ; les quatre catalogues traduits.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la sous-fiche
`_bmad-output/implementation-artifacts/v016-2b-releve-manuel-ecran.md` — **AC-19 à AC-27, AC-29-bis, AC-30**, *Tests et
mutations prévus*, *À recontrôler à l'étape 0*, le **Dev Agent Record** et le **Change Log** ; la fiche parente
`v016-2-releve-manuel.md` (§ *Le corps de requête*, *Réponse de l'aperçu*, *Codes d'erreur*, *Ensembles fermés*, *Ordre
des refus*, *Ce que les gardes ne voient pas*) ; le registre `v016-2-choix-autonomes.md`, **C-v016-2-24 à 27** ; la
contrainte `/home/gcorbaz/devel/kesh-gate-logs/prompts/v017-donnees-a-garder.md`. Le Dev Agent Record déclare des rouges,
des gates, des mutations, un E2E et des captures : **ne les prends pas pour argent comptant**, recoupe-les avec les
journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/v0162b/` (rouges, gates, `mutations/`, E2E,
`captures/` — regarde les images).

⛔ Ne lis rien sous `~/kesh-test/` (données réelles).

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, nulle
  part ; **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`) : compare par des commandes enchaînées (pipe). Un
  fichier créé par erreur se **déclare** au rapport, il ne se supprime pas.
- **Aucun `rm`, sous aucune forme** ; ni **`pkill`**, ni **`killall`**.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git restore`), aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni **`python3`**, ni `perl`, ni `node`/**`npm`**/
  `npx`, ni `bash -c`, ni `jq` ; ni **`scripts/prepare-release.sh`**, ni `scripts/test-fast.sh`, ni
  `scripts/mem-guard.sh`, ni **aucun script du dépôt** ni de `kesh-gate-logs` ; ni **`make`**, ni `latexmk`, ni
  **`cargo`**, ni **`docker`**, ni `mariadb`, ni `sqlx`, ni **`curl`**, ni **`claude`** sous aucune forme, ni
  **`sed -i`**, ni `playwright`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`, `-delete`), `wc`, `sort`, `uniq`, `cmp`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '` et `git show <réf>:<f>.pdf | pdftotext - - | tr '\n' ' ' | tr -s ' '`.
- ⛔ **Sur le PDF aplati, jamais de `grep -oE`/`grep -E` à quantificateur `{0,N}`** : `grep -cF` ou `grep -oF … | wc -l`
  sur des motifs fixes, sous `timeout 60`. Le PDF rend l'apostrophe en `’` (U+2019).
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée.

## Les axes obligatoires, pour chaque lentille

1. **Le manuel utilisateur (`.tex` et PDF aplati) contre le code** : chaque promesse du § *Saisir un relevé à la main*
   (pas d'« ignorer » ; équilibre exact sans confirmation ; confirmation du solde de départ, jamais bloquant ; relevé
   identique — quels champs entrent réellement dans l'empreinte, `manual_statement_canonical_bytes` ? — ; suppression
   puis ressaisie ; conduite « la banque passe au CAMT », contre `kesh-core/src/bank_imports.rs` (clé de doublon) ;
   limites : CHF, 500 lignes, compte clos, relevé précédent **à solde** finissant **avant**). Le filet des phrases
   anciennes : `Seul le CAMT`, `Seuls les relevés`, `deux formats`, `n’en a`, `transactions importées`, `le nom du
   fichier`, `réimport` — chaque occurrence restante est-elle juste d'un relevé saisi ? Le PDF dit-il la même chose que
   le `.tex` ? `admin-manual.tex:82` et son PDF.
2. **Les gardes i18n** : `REPLIS_A_SITE_UNIQUE` (G13, `i18n-repli-divergent-actif.test.ts`) — chaque clé statique neuve
   `bank-import-manual-*` (hors `-reason-`) et `bank-import-detail-*` y est-elle, et les quatre clés récrites ? le
   décompte « quatre-vingt-onze » se recompte-t-il ? ; `i18n-keys.test.ts` (`sitesTotal` 2711, `sitesGabarit` 12,
   `SITES_GABARIT_ATTENDUS`, `MOTIFS_DYNAMIQUES`, `CARDINALITES`) recomptés par la commande que le commentaire cite ;
   `i18n-libelle-en-dur.test.ts` (76 candidates, `conforme` 62) ; parité des quatre catalogues ; G8/G8-bis (la boîte de
   clôture garde-t-elle « rouvrir » ?) ; les traductions de-CH, it-CH, en-CH disent-elles la même chose que le fr-CH ?
3. **Le test croisé des `reason` (G24)** : l'extracteur peut-il passer à vide, ou laisser passer une clé sur une ligne de
   continuation ? L'égalité est-elle vraiment dans les deux sens ? Repose-t-il sur l'exhaustivité non circulaire de `ALL`
   (C-v016-2-20, tests de `kesh-core`) ? La table TypeScript respecte-t-elle la forme imposée (un objet, une entrée par
   ligne, sans commentaire) ?

## Lentille B — Blind (le diff seul, sans la fiche)

1. **Le code de production** : `manual-statement.ts` (Big.js partout ? `isIsoDate`, comptage en points de code après
   `trim`, normalisation `toFixed(2)`, `screenRowOf` aux bornes, `describeManualError` pour chaque code) ;
   `ManualStatementForm.svelte` (les deux `$effect` peuvent-ils boucler ? `seq` et clé d'instantané ; `fieldset
   disabled` ; garde `submitting` ; un 422 de création retire-t-il bien l'aperçu ? clés `{#each … (line)}` sur des
   proxys ; `remove` pendant un envoi) ; `isValidAmount` (défauts du journal inchangés ? l'expression construite par
   `RegExp` est-elle sûre ?) ; `DeleteBankImportAction.svelte` et le détail.
2. **Les tests** : chacun prouve-t-il ce qu'il nomme, ou peut-il passer à vide ? (M29 : la saisie dans un `fieldset`
   désactivé atteint-elle vraiment l'état dans jsdom ?) L'E2E : dates locales de Zurich, compte et IBAN, rapprochement
   et journal d'audit isolés par l'identifiant de l'import.
3. **Les textes** : CHANGELOG (*Ajouté* en tête des sections de `[0.17.0] — Non publié`, `refs` par lien, aucun mot-clé
   de fermeture), README, aucun mot-clé de fermeture dans `git log 285d2fde..HEAD --format=%B`.

## Lentille E — Edge cases

1. **La saisie** : soldes `-0`, `"1 000"`, `"1'000"`, `"1000,"`, quatorze chiffres, trois décimales ; un montant `0.00` ;
   une ligne faite de blancs au milieu ; 500 et 501 lignes ; une période d'un jour ; un 1ᵉʳ janvier ; le retrait d'une
   ligne pendant un aperçu ; un compte clos entre l'aperçu et l'enregistrement ; un doublon apparu entre l'aperçu et
   l'enregistrement ; un `400 VALIDATION_ERROR` **avec** `field` ; une ligne à deux montants envoyée par « Vérifier ».
2. **L'affichage** : un import `MANUAL` sans numéro, avec un numéro de 255 caractères ; un CSV dont le `filename`
   commence par `saisie-manuelle-` ; la notification de suppression ; un Consultation sur le détail d'un relevé saisi.

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC (**AC-19 à AC-27, AC-29-bis, AC-30**), « tenu », « non tenu » ou « non vérifiable en lecture seule », avec
la preuve. L'étape 0 : point bloquant vérifié au code commité (ligne citée), renvois P8 appliqués ? Les mutations M13 à
M30 : chacune tuée **par le test nommé** (journaux `mutations/`) ? Les compteurs recalés se recomptent-ils ? Les choix
**C-v016-2-24 à 27** contredisent-ils un AC ? Chaque nombre du Dev Agent Record (Vitest, nextest, E2E et échecs
nominatifs contre `docs/testing.md` § *Les échecs attendus*) se recompte-t-il depuis sa source, avec son périmètre ? Les
captures couvrent-elles l'énumération de l'AC-25 (et les manques sont-ils déclarés) ? `sprint-status.yaml` : `review`.

## Rapport

Écris ton rapport dans **un seul** fichier : `/home/gcorbaz/devel/kesh-gate-logs/v0162b-review-p1-B.md` (lentille B),
`/home/gcorbaz/devel/kesh-gate-logs/v0162b-review-p1-E.md` (lentille E) ou
`/home/gcorbaz/devel/kesh-gate-logs/v0162b-review-p1-A.md` (lentille A) — par l'outil d'écriture de fichier, **jamais**
par une redirection. Contenu :

1. **Les findings** : identifiant (`B1`, `E1`, `A1`…), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier et ligne,
   la preuve (sortie copiée), la correction proposée, et s'il est **né du développement** de la v016-2b ou **antérieur**.
2. **Les axes réellement exercés, et ceux qui ne l'ont pas été** — sans cette liste, un « 0 finding » ne compte pas.
3. **Le décompte** `CRITICAL / HIGH / MEDIUM / LOW`, recompté depuis ta liste.
4. Les fichiers créés par erreur, et toute commande lancée hors de la liste permise, s'il y en a.
