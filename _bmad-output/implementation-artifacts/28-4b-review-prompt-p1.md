# Prompt — revue de code P1, Story 28-4b (clore un compte bancaire, partie écran)

*Versionné le 2026-10-13. Trois lentilles (**Sonnet** — la 28-4a, partie serveur, a été revue par Opus ; rotation
Sonnet ↔ Opus pour les passes complètes), contexte frais chacune, lecture seule.*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-28-4b`, branche `story/28-4b`.
**Le diff à revoir** : `git diff 4687af69 b32390f9 -- . ':(exclude)_bmad-output'` (`4687af69` = `origin/main`, qui
porte la 28-4a et la 28-5b ; `b32390f9` = **dernier commit de code** ; le commit suivant, `4b45ec1b`, ne porte que les
PDF régénérés, et les commits qui suivent la fiche, le registre des choix, le registre de sprint et ce prompt). Les
PDF se contrôlent à `4b45ec1b`. ⚠️ Pathspec `':(exclude)…'` et **non** `':!…'`. Fiche :
`_bmad-output/implementation-artifacts/28-4-cloture-compte-bancaire.md` — **partie b seulement** : AC11 à AC17 ;
tâches T10-T15 ; Dev Notes § *Tests* (T-b1 à T-b14) et § *Mutations* (M25 à M32) ; et le **Dev Agent Record**,
section *28-4b* (campagne de mutations, gates, décomptes, captures). Choix C-28-4-36 à C-28-4-46 au registre
`epic-28-choix-autonomes.md` (nés du développement de la 28-4b ; C-28-4-1 à 35 sont ceux de la fiche et de la 28-4a).
Règles : `CLAUDE.md` du dépôt — § « Recompter ses propres comptes rendus », § « Propagation post-patch », § « Le
prompt d'une passe doit NOMMER le manuel », § « Une passe qui ne déclare pas ses axes exercés ne compte pas », § *Test
Locally First* (E2E). Issues : #642 (la PR de la 28-4b la **ferme** — `closes #642` dans le corps de la PR, jamais
dans un commit) et #651 (L1, `refs` seulement).

⛔ **DONNÉES PERSONNELLES — DÉPÔT PUBLIC.** N'ouvre aucun fichier hors du dépôt et de
`/home/gcorbaz/devel/kesh-gate-logs/` ; rien sous `~/kesh-test/`. Les captures
`kesh-gate-logs/28-4b-captures/*.png` sont synthétiques (IBAN de test) : tu peux les ouvrir.

## Axes communs aux trois lentilles (chacune les déclare exercés ou non)

1. **L'écran contre l'AC11.** `frontend/src/routes/(app)/bank-accounts/+page.svelte` : bascule des clos (défaut
   masqué, rechargement avec `includeClosed`), ligne close (badge daté au format suisse, **seule** action *Rouvrir*,
   Lier / Modifier / Archiver / Clore absents), principal (*Clore* désactivé, `title` et texte d'aide), dialogue de
   clôture (date **locale** préremplie — `todayLocal`, jamais l'UTC —, avertissement d'un solde non nul et non `null`,
   *Annuler* `outline`, *Clore le compte* `destructive` sans classe), dialogue de réouverture, refus serveur par
   `errorMessageOf` (`[object Object]` interdit), toasts. Un compte **archivé et clos** (base restaurée) : que montre
   la ligne ? La combinaison des deux bascules ? L'atténuation déplacée sur les cellules (C-28-4-46) change-t-elle le
   rendu d'une ligne **archivée** ?
2. **L'API frontend et ses appelants (AC11, AC12).** `bank-accounts.api.ts` : `closedOn` (obligatoire dans le type,
   `?? null` au brut), `listBankAccounts(includeArchived, includeClosed)` — les **quatre** URL ; **tous** les
   appelants existants gardent-ils leur sens (`grep -rn "listBankAccounts(" frontend/src`) ? Seul
   `settings/invoicing` passe `(false, true)` ; est-ce le seul qui en a besoin (C-28-4-7 : un compte clos reste lié à
   son compte comptable pour le serveur) ? `closeBankAccount` / `reopenBankAccount` : méthode, chemin, corps.
3. **i18n (AC15).** Seize clés × quatre locales, recompte par `grep -c` ; vocabulaire allemand de C-28-4-32
   (« aufgelöst », « auflösen », « wieder eröffnen » — jamais « geschlossen / schliessen », G11) ; replis Svelte
   **égaux** aux valeurs fr-CH ; les quatre exemptions de G8 / G8-bis (`kesh-i18n/src/loader.rs`, C-28-4-39) sont-elles
   justes, et aucune autre clé neuve n'aurait-elle dû y entrer ? `settings-bank-manage-hint` (quatre locales et repli
   de `settings/+page.svelte`) ; `sitesTotal` 2103 (2084 + 19, mesuré par la collecte) et son commentaire.
4. **Les tests prouvent-ils ce qu'ils nomment.** T-b1 à T-b14 contre la fiche ; la campagne M25-M32 (journal
   `kesh-gate-logs/28-4b-mutations.log`, script cité au Dev Agent Record) : cherche une **mutation plausible qui
   resterait verte** (ex. : le badge sans date ; le toast « masqué » toujours affiché ; la date envoyée en UTC ;
   `includeClosed` passé à la bascule des **archivés** ; un dialogue de clôture qui s'ouvre sur le principal par une
   autre voie ; `reopenBankAccount` qui envoie `closedOn`). L'E2E `bank-accounts-closed.spec.ts` : T-b12 prouve-t-il
   l'absence **et** la présence dans chacun des six sélecteurs et la tuile ? La preuve « par la réponse consommée »
   pour l'échéancier et la fiche fournisseur (C-28-4-36) tient-elle — le `waitForResponse` peut-il capter une autre
   requête `GET /api/v1/bank-accounts` que celle de l'écran visé, ou passer à vide ?
5. ⛔ **LA DOCUMENTATION — MANUELS (PDF compris), `docs/api-external.md`, CHANGELOG.** Chaque phrase neuve ou
   modifiée dit-elle vrai **au code** (serveur de la 28-4a compris) ? Au manuel utilisateur : § *Clore un compte
   bancaire* (trois conditions, ce qui devient impossible, ce qui reste, solde, réouverture, deux limites dont #651,
   clôture et archivage — « un compte archivé ne se rouvre pas », « le principal s'archive quand tous les autres sont
   archivés ou clos », « Paramètres affiche Aucun compte bancaire configuré ») ; l'import « ni archivés ni clos » ;
   § *Supprimer un import* (ordre des refus de W24) ; § *Solde comptable* ; règlements client et fournisseur
   (archivé, clos) ; la voie *Annuler le rejet*. Au manuel d'administration : § *Passer à la 0.13.3*, étape 1 de la
   procédure, § *Rollback*, § *Comptes débiteurs et créanciers* (« clos compris »). Cherche aussi une phrase des
   manuels, du README ou de `website/` que la clôture rend **fausse** et que la story n'a pas touchée
   (`grep -n "archiv" docs/manual/fr/*.tex`, sélecteurs, tuile, Paramètres). Contrôle les **PDF aplatis** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers
   `/home/gcorbaz/devel/kesh-gate-logs/28-4b-review-p1-<lettre>-user-manual-flat.txt` (idem `admin-manual`) —
   ⚠️ `pdftotext` rend « ﬀ » (ligature) et « ’ » (apostrophe typographique). `docs/api-external.md` : note ⁹, les
   sous-sections gardées (règlements, factures fournisseurs, acceptation, annulation d'un rapprochement, annulation
   d'un rejet, import), les cinq codes du § 10 ; l'ordre des refus annoncé (« avant les motifs qui suivent ») est-il
   vrai dans **chaque** chemin (W25, W28, W29, W9) ? CHANGELOG : une seule section `[0.13.3]`, bloc *À faire*,
   l'entrée « Ajouté » de #642 à côté de celle de #526.
6. **Gardes et décomptes.** `boutons-rouges-inventory.test.ts` (garde f), `erreurs-en-texte.test.ts`,
   `e2e-selecteurs-traduits.test.ts`, `lint-i18n-ownership`, G18 et G9 (`textes_coherents.rs`). **Recompte depuis la
   source** tout décompte du Dev Agent Record (tests neufs par fichier aux **deux** bornes `4687af69` / `b32390f9`,
   fichiers au `git diff --stat`, mutations) et vérifie le **périmètre** déclaré de chaque nombre.

## Lentilles

- **B — Blind Hunter : bugs et états**, sans présupposer que la fiche a raison. Les états de la page (`mode`,
  `submitting`, `closeError` / `reopenError` remis à zéro ?) ; double clic sur *Clore le compte* ; la version envoyée
  (lue de `bankAccounts` au clic : périmée après un autre onglet ?) ; date vidée (bouton désactivé) ; date invalide
  ou hors `1000..=9999` saisie à la main (le `400` s'affiche-t-il ?) ; fuseau de `todayLocal` ; `Big` sur un
  `currentBalance` malformé (exception non attrapée dans le rendu ?) ; la bascule pendant un dialogue ouvert ; le
  toast quand les clos sont affichés ; la réouverture d'un compte qui ne figure plus dans la liste.
- **E — Edge Case Hunter : bords, et les tests prouvent-ils ce qu'ils nomment.** Rejoue mentalement la campagne et
  cherche la mutation verte (axe 4) ; les mocks de `bank-accounts-page.test.ts` (interpolation du repli ajoutée : un
  test existant change-t-il de sens ?) ; les quatre gabarits de test complétés par `closedOn: null` ; T-b12 à vide
  (sélecteur absent → `toHaveCount(0)` passe !) — chaque assertion d'absence est-elle précédée d'une assertion de
  présence sur le même sélecteur ? Le montage E2E (comptes 1021 / 1022 créés, IBAN de test, idempotence si la spec
  est rejouée sans reseed, état laissé pour les specs suivantes — compte rouvert, écriture de 42.50 au compte 1022).
- **A — Acceptance Auditor : conformité, décomptes, documentation.** Chaque AC de la partie b (AC11 à AC17) contre le
  code : satisfait, partiel, absent ; les **legs de la 28-4a** (fiche, Dev Agent Record de la 28-4a :
  `user-manual.tex` « non archivés », textes de C-28-4-27 et C-28-4-32, `docs/api-external.md` pour `closedOn` et
  les routes, la limite #651 au manuel) sont-ils tous repris ? Les captures (AC17) existent-elles pour les quatre écrans
  demandés, clair et sombre — et que vaut le « sombre » (Kesh n'a pas de thème sombre, C-28-1-7) ? Les chiffres de
  gates et d'E2E déclarés correspondent-ils aux journaux (`kesh-gate-logs/28-4b-final-2*`) ? Les choix C-28-4-36 à
  46 : justifiés, ou défaut ? README et `website/` : une ligne à compléter a-t-elle été manquée ?

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/28-4b-review-p1-<B|E|A>.md` (ta lettre). Findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code relu), correction proposée. Pour
tout finding affirmant qu'un code ou un texte est absent ou présent : la sortie d'un `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** (les six axes communs et ceux de ta lentille) — un « 0 finding » sans
elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/` (et les textes aplatis des
PDF au même endroit) ; aucune commande qui écrit dans le dépôt ou dans une base, ni qui compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`, `latexmk`,
`xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`reset`/`rebase`/`restore`/`push`/`tag`, `sqlx`, `cargo`
(aucune sous-commande), `npm`, `npx`, `node`, `deno`, `bun`, `python`/`python3` (pas même le script de mutations),
`perl`, `ruby`, `php`, `sed -i`, `awk` en écriture, tout autre interpréteur, `mariadb`, `mysql`, `docker`, `curl` et
`wget`, `gh` en écriture, `pkill`/`kill`, aucune requête SQL. Aucun fichier hors du dépôt. Autorisés : lecture,
`grep`, `sed -n`, `wc`, `sort`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers
`/home/gcorbaz/devel/kesh-gate-logs/`, l'ouverture des PNG de `kesh-gate-logs/28-4b-captures/`.
