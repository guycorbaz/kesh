# Prompt — revue de code P2, Story 15-1c-ii (le lettrage dans le reste de Kesh — fiche d'écriture, Grand livre, manuel, CHANGELOG)

*Versionné le 2026-10-10. Trois lentilles (Opus), contexte frais chacune, lecture seule. Passe COMPLÈTE : la
remédiation P1 a touché du code de production.*

⚠️ **La branche a été rebasée** sur `origin/main` `66935feb` (la 15-1c-i fusionnée en squash ; arbre identique à
`011ea618`). SHAs après rebase : développement **`2fc1c784`** (ex-`d7437b38`), prompt P1 `317c6777`, remédiation P1
**`d4524a7d`**. Les rapports P1 (`kesh-gate-logs/15-1c-ii-review-p1-{B,E,A}.md`) et le Change Log « Revue de code P1 » de la
fiche disent ce qui a été corrigé. **Relis d'abord la remédiation** (`git show d4524a7d`) : le motif mesuré du projet est
que la sévérité se déplace vers ce qu'on vient d'écrire.

Dépôt (worktree) `/home/gcorbaz/devel/kesh-15-1c-ii`, branche `story/15-1c-ii-lettrage-dans-kesh`.
**Le diff à revoir** : `git diff 66935feb d4524a7d` (développement et remédiation P1 ; `HEAD` ne porte en plus que ce prompt). Fiche :
`_bmad-output/implementation-artifacts/15-1c-ii-lettrage-dans-kesh.md` — AC9, AC11 part ii, AC12 (points 1 à 4), AC13
part ii, AC14, AC17, Tasks, « Tests de T6 » (6 tests numérotés), section « Reçu de la 15-1c-i », Dev Notes, et le **Dev
Agent Record**, qui déclare cinq choix (C-15-1c-ii-1 à 5, registre `epic-15-choix-autonomes.md`), onze mutations, un
inventaire de 75 sites du manuel (18 réécrits, 57 justes) et des décomptes. Fiches à connaître : `15-1c-i-ecran-postes-ouverts.md`
(ce que l'écran fait réellement : la section neuve du manuel le décrit), `15-1c-proposition-ecran.md` (registre des
C-15-1c-*). Règles : `CLAUDE.md` du dépôt — § « Recompter ses propres comptes rendus », § « Propagation post-patch — grep
du symptôme », § « Le prompt d'une passe doit NOMMER le manuel », § « Un appariement automatique propose, il ne crée
jamais ». Issues : #518 (la PR la fermera), #607 (points 1 et 2, la PR la fermera — `gh issue view 607`). Hors périmètre :
#602, #606.

**Contexte** : story **frontend + documentation**, aucun code serveur. Code : `routes/(app)/journal-entries/[id]/+page.svelte`
(colonne « Lettrage », pied *Total*, lien du motif `ENTRY_LETTERED`) et son test neuf `journal-entry-page.test.ts` ;
`features/reports/GeneralLedgerView.svelte` (table `COLUMNS`, `colspan` calculés, colonne « Lettrage », lien « Postes
ouverts de ce compte ») et son test ; `features/reports/reports.types.ts` (`LedgerLine.letteringCode`) ;
`routes/(app)/reports/+page.svelte` (`letterableAccountIds`) ; `features/open-items/open-items.ts` (`openItemsHref`,
`groupHref` réécrit) ; 3 clés ×4 locales ; bornes de deux gardes i18n ; E2E `frontend/tests/e2e/open-items.spec.ts`
(scénarios 7 à 9, `monter()` étendu). Texte : `docs/manual/fr/user-manual.tex` (section neuve « Lettrage et postes
ouverts », propagation), `admin-manual.tex`, `marketing-brochure.tex` **et leurs trois PDF**, `CHANGELOG.md` (entrée
*Ajouté* fondue), `docs/api-external.md`, `README.md`, `website/roadmap.html`, `website/index.html`.

⛔ **Axes communs aux trois lentilles** (chacune les déclare exercés ou non) :
1. **`colspan`** : pour **chaque** `<tr>` du corps et du pied du Grand livre (ouverture, rupture, mouvement, ligne vide,
   total des mouvements, clôture) et de la fiche d'écriture (lignes, pied *Total*, avec et sans projets), la somme des
   `colspan` égale le nombre d'en-têtes, **et** chaque montant tombe sous son en-tête (solde sous « Solde progressif »,
   totaux sous « Débit »/« Crédit »).
2. **Le lien « Postes ouverts de ce compte »** : présent ssi le compte est lettrable (jamais un lien qui rendrait 409),
   absent si la liste des comptes a échoué ; `asOf` = fin de la période affichée ; paramètres que lit réellement l'écran
   (`parseScreenState`).
3. **Le lien du motif `ENTRY_LETTERED`** : sur ce motif seul (C-15-1c-9), jamais sur un autre motif étiqueté ; le toast
   d'un refus reste un texte ; Consultation ne voit aucun motif ; le motif de modification n'est-il jamais masqué par la
   règle `showModificationReason` quand le code est `ENTRY_LETTERED` ?
4. **i18n des quatre locales** : 3 clés, vocabulaire C-15-1a-i-4 (de *Ausgleich*, en *matching*, it *abbinamento*),
   repli Svelte = valeur fr-CH, un seul repli par clé ; `columnLabel` et les huit clés d'en-tête existantes inchangées.
5. ⛔ **LE MANUEL — sources ET PDF.** La section neuve dit-elle **vrai** sur ce que le code fait **réellement** (relire
   `features/open-items/*.svelte`, `open-items.ts`, `open-items-labels.ts`, `crates/kesh-core/src/lettering/proposals.rs`,
   `crates/kesh-api/src/routes/letterings.rs`, les codes de `crates/kesh-api/src/errors.rs`) : chaque affirmation —
   bornes 2 et 200, case à cocher, bouton « Lettrer » et ses conditions, sélection hors page, ordre des propositions,
   « au X n'est pas un instantané », trois refus du délettrage, règle des périodes (C105), rôles, exports, frontière
   avec la réconciliation, audit. Les **sites réécrits** (sixième condition de *Modifier*, #607 aux lignes du « grand
   livre la montre soldée », paiement détaché, FAQ, glossaire, manuel d'administration — dont « le grand livre n'existe
   pas encore ») et les **sites déclarés justes** : en reprendre un échantillon d'au moins dix et juger. **Cherche une
   phrase du manuel que l'écran rend fausse et que l'inventaire n'atteint pas** (le motif qui a valu le reçu A-1 : une
   énumération du menu). Contrôle les **PDF** aplatis (`pdftotext docs/manual/fr/<f>.pdf - | tr '\n' ' ' | tr -s ' '`
   vers `/home/gcorbaz/devel/kesh-gate-logs/`) : section neuve présente, textes provisoires absents, brochure sans
   lettrage au backlog — attention à la césure qui coupe un mot. Références croisées (`\ref`) toutes définies. Numéros
   de compte cités en exemple : existent-ils sous leur nom (G4-bis) ?
6. **Aucune promesse au-delà du code** : CHANGELOG, README, site, brochure, `api-external.md` — rien qui dise le
   lettrage absent, futur ou « par l'API seulement », et rien qui promette un appariement automatique (Kesh
   **propose**) ni une fonction non livrée.

## Lentilles

- **B — Blind Hunter : bugs et états**, sans présupposer que la fiche a raison. `GeneralLedgerView.svelte` : la table
  `COLUMNS` et les constantes dérivées (une colonne ajoutée demain, ou déplacée, laisse-t-elle un `colspan` faux sans que
  rien ne rougisse ?), `{#each { length: N }}` (N = 0, clés), le lien dans un en-tête `flex-wrap`, compte archivé,
  `letterableAccountIds` par défaut. La page des rapports : `$derived` sur `accounts`, chargement tardif de la liste
  (le Grand livre affiché avant la liste : le lien apparaît-il ensuite ?). La fiche : `letteredBy` quand le libellé est
  nul, `showModificationReason`, mode édition. `openItemsHref`/`groupHref` : sortie identique à l'ancien `groupHref`
  pour tous les appels de la 15-1c-i (relire ses sites). Types contre le DTO réel (`kesh-report/src/general_ledger.rs`).
- **E — Edge Case Hunter : bords, et les tests prouvent-ils ce qu'ils nomment.** Section sans mouvement, section à une
  rupture, plusieurs sections dont une seule lettrable, période d'un jour, écriture sans ligne lettrée, toutes lettrées,
  projets. **Rejoue mentalement les mutations M1 à M10** du Dev Agent Record (dont M1, déclaré équivalent) et cherche une
  **mutation plausible qui resterait verte** (ex. : lien sur `dto.period.from` ; lien sur tout compte d'actif ;
  cellule « Lettrage » rendue sur la rupture mais pas sur la clôture ; motif `ENTRY_LETTERED` avec libellé nul ;
  colonne « Lettrage » de la fiche avant « Débit »). Le spec E2E : scénarios (7) à (9) indépendants, `data-testid`
  seuls (garde #326), nettoyage qui ne masque pas un échec, date du Grand livre (`todayZurich()` et l'année), regex
  d'URL, compte créé au bon numéro.
- **A — Acceptance Auditor : conformité, décomptes, documentation.** AC9, AC11 part ii, AC12 (points 1 à 4 — la liste
  des sujets de la section neuve au point 1, un par un), AC13 part ii, AC14 (une entrée *Ajouté* cohérente ; *Modifié*
  relues ; aucun fait des cinq anciennes entrées perdu — compare `git show 66935feb:CHANGELOG.md`), AC17 (les quatre
  supports). « Reçu de la 15-1c-i » points 1 et 2. Les tests 1 à 6 de T6 existent-ils et prouvent-ils ce qu'ils nomment.
  **Décomptes recomptés depuis la source** (75 sites = 18 + 57 et leur liste ; lignes « lettr » après ; Vitest
  1270 → 1287 après P1 (fiche 7 → 8, Grand livre 8 → 15, `open-items.test.ts` +2) ; `sitesTotal` 2017 → 2020 par fichier ; « libellé en dur » 60 → 61 ; `grep -c '518'
  CHANGELOG.md` 12 → 8). Les cinq choix C-15-1c-ii-1 à 5 : justifiés, ou défaut ? #607 : les deux points sont-ils
  réellement traités ?

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/15-1c-ii-review-p2-<B|E|A>.md` (ta lettre). Findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code relu), correction proposée. Pour tout
finding affirmant qu'un code ou un texte est absent ou présent : la sortie d'un `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** (les six axes communs et ceux de ta lentille) — un « 0 finding » sans elle ne
compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/` (et le texte aplati des PDF au même
endroit) ; aucune commande qui écrit dans le dépôt ou dans une base, ni qui compile ou exécute : `scripts/*` (dont
`scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`, `latexmk`, `xelatex`, `git commit`/`add`/
`checkout`/`switch`/`stash`/`reset`/`rebase`, `sqlx`, `cargo` (aucune sous-commande), `npm`, `npx`, `node`, `docker`, `gh` en
écriture, aucune requête SQL ni HTTP. Autorisés : lecture, `grep`, `sed -n`, `wc`, `git log`/`show`/`diff`, `gh issue view`,
`pdftotext` vers `/home/gcorbaz/devel/kesh-gate-logs/`.
