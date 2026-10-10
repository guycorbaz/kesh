# Prompt — revue de code P1, Story 15-1c-i (l'écran des postes ouverts — consulter, lettrer, délettrer)

*Versionné le 2026-10-10. Trois lentilles (Sonnet), contexte frais chacune, lecture seule.*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-15-1c-i`, branche `story/15-1c-i-ecran-postes-ouverts`.
**Le diff à revoir** : `git diff 4d5fd06c be6afc26` (le commit de développement ; `4d5fd06c` est `origin/main`, la 15-1c-0
fusionnée ; `HEAD` ne porte en plus que ce prompt). Fiche : `_bmad-output/implementation-artifacts/15-1c-i-ecran-postes-ouverts.md`
— « Contrats consommés », AC1–AC8, AC10, AC11 part i, AC13 part i, AC19, Tasks, « Tests de T8 » (12 tests numérotés), Dev
Notes, et le **Dev Agent Record**, qui déclare six choix (C-15-1c-i-1 à 6, registre `epic-15-choix-autonomes.md`), douze
mutations et des décomptes. Fiches à connaître : `15-1c-proposition-ecran.md` (l'index, registre des C-15-1c-*), et
`15-1c-ii-lettrage-dans-kesh.md`, pour savoir ce qui **n'appartient pas** à cette story (Grand livre, fiche d'écriture,
manuel, entrée *Ajouté* du CHANGELOG, `closes #518`). Le contrat serveur livré se lit dans
`crates/kesh-api/src/routes/letterings.rs` (DTO) et `crates/kesh-api/src/errors.rs` (codes). Règles : `CLAUDE.md` du dépôt —
§ « Recompter ses propres comptes rendus », § « Le prompt d'une passe doit NOMMER le manuel », § « Un appariement
automatique propose, il ne crée jamais ». Issue : #518 (la story ne la ferme pas). Hors périmètre : #602, #606, #607.

**Contexte** : story **frontend + i18n**, aucun code serveur. `frontend/src/lib/features/open-items/` (neuf) :
`open-items.ts` (logique pure : URL, liens de pièce, somme `big.js`, état du bouton, refus périmé, sens),
`open-items-labels.ts` (textes), `open-items.api.ts`, `open-items.types.ts`, `OpenItemsScreen.svelte` (l'écran ; l'URL est
la source de vérité), `OpenItemsTable.svelte`, `ProposalsPanel.svelte`, `LetteringGroupPanel.svelte`, `DocumentCell.svelte`,
`LetteringCodeLink.svelte` ; route `routes/(app)/open-items/+page.svelte` ; menu `routes/(app)/+layout.svelte` ;
`accounts.types.ts` (`letterable`) ; 73 clés ×4 locales (`crates/kesh-i18n/locales/*/messages.ftl`) ; bornes de trois gardes
i18n ; E2E `frontend/tests/e2e/open-items.spec.ts` ; `README.md`.

⛔ **Axes communs aux trois lentilles** (chacune les déclare exercés ou non) :
1. **Aucun lettrage sans clic** : aucun chemin n'appelle `createLettering` hors d'un geste (montage, effet, rendu,
   rechargement après refus) ; une proposition envoie exactement ses deux `lineId`.
2. **Tout 404/409 périmé** (C-15-1c-17, C-15-1c-24) : liste **et** propositions relues, sélection vidée, pour le `POST`
   (manuel et proposition) et le `DELETE` ; un 400 garde la sélection.
3. **L'URL** : `asOf` toujours envoyé, date **locale**, écrite par remplacement ; effets Svelte sans boucle (`untrack`),
   réponses tardives ignorées (numéros de requête), changement de compte / de date / de groupe.
4. **Arithmétique `big.js`** : aucune somme ni comparaison de montant en flottant là où elle décide (bouton, égalité du pied,
   sens) ; `Number(v) === 0` n'est employé que pour masquer un zéro à l'affichage — est-ce sûr ?
5. **Accessibilité** : cases étiquetées, motif d'absence de case lisible au lecteur d'écran, `role="alert"/"status"`,
   bouton inactif relié à son motif (`aria-describedby`), en-têtes `scope`, `<label>` des champs.
6. **i18n des quatre locales** : 73 clés présentes et traduites dans les quatre, vocabulaire C-15-1a-i-4 (de *Ausgleich*,
   en *matching*, it *abbinamento*, vouvoiement pluriel ; *Abgleich / reconciliation / riconciliazione* réservés au
   rapprochement bancaire), variables Fluent identiques entre locales et aux arguments des sites, repli Svelte = valeur
   fr-CH (G13), un seul repli par clé, aucune expression de sélection.

## Lentilles

- **B — Blind Hunter : bugs et états**, sans présupposer que la fiche a raison. `OpenItemsScreen.svelte` geste par geste :
  `applyState` (premier passage, `asOf` nul, compte nul, `group` seul), `letter`, `dissolve` (ordre `reloadListAndProposals`
  / `navigate`, panneau, message), `busy`, double clic, `SvelteMap` et dérivés (`selected`, `offPage`), pagination (`offset`
  non dans l'URL ; que devient la page après un rechargement ?), effet de chargement des comptes. Liens (`documentHref`,
  `groupHref`, `LetteringCodeLink`) ; `isIsoDate` (années, fuseaux) ; `todayLocal` calculé une fois au montage (écran ouvert
  à minuit ?). Types contre les DTO réels (`letterings.rs`).
- **E — Edge Case Hunter : bords, et les tests prouvent-ils ce qu'ils nomment.** Page vide, dernière page, 0 compte
  lettrable, compte archivé, compte devenu non lettrable (409) avec groupe ouvert, groupe `document` sans numéro, facture
  fournisseur sans numéro, règlement sans facture, `amountDue` nul, `letteredOn` nul, code saisi en minuscules ou avec
  espaces, `openTotal ≠ balance`. **Rejoue mentalement les douze mutations** du Dev Agent Record et cherche une **mutation
  plausible qui resterait verte** (ex. : sélection non vidée au changement de date ; propositions non relues après un
  succès ; 404 du `POST` affiché par le message serveur ; case présente en Consultation ; `asOf` omis à la pagination ;
  `letterable` ignoré au sélecteur). Le spec E2E : sélecteurs `data-testid` seuls, montage autonome (compte créé, numéro
  ≤ 10 caractères), scénarios (1) à (6) conformes à AC13.
- **A — Acceptance Auditor : conformité, décomptes, documentation.** AC1 à AC19 point par point contre le code (dont la
  table des liens d'AC2, les six libellés d'AC3, l'ordre des motifs d'absence de case et des messages du bouton d'AC4, la
  phrase d'AC8 au mot près, AC10 — ni case, ni « Lettrer », ni « Délettrer », **sans motif** en Consultation). Les tests
  1 à 12 existent-ils et prouvent-ils ce qu'ils nomment. **Décomptes recomptés** depuis la source (86 tests en 5 fichiers :
  16/11/13/13/33 ; 73 clés par locale ; `sitesTotal` 1923 → 2015 et sa ventilation par fichier ; garde « libellé en dur »
  47 → 60, `ecartee` 14, `conforme` 46 ; garde « un repli par clé » 214 → 289). Les six choix C-15-1c-i-1 à 6 :
  justifiés, ou défaut ? README (AC19, ligne v0.13.0). **Manuel** : la story ne le touche pas (la 15-1c-ii le fera) —
  vérifie qu'aucun texte du manuel n'est rendu **faux** par l'écran (`grep -n -i "lettr\|postes ouverts" docs/manual/fr/*.tex`
  et le PDF aplati, `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers
  `/home/gcorbaz/devel/kesh-gate-logs/`) — ex. « le délettrage se fait par l'API dans cette version » : faux désormais ? Si
  oui, la fiche dit-elle qui le corrige et quand (Status, « pas de tag entre i et ii ») ?

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/15-1c-i-review-p1-<B|E|A>.md` (ta lettre). Findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code relu), correction proposée. Pour tout
finding affirmant qu'un code est absent ou présent : la sortie d'un `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** (les six axes communs et ceux de ta lentille) — un « 0 finding » sans elle ne
compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/` (et le texte aplati des PDF au même
endroit) ; aucune commande qui écrit dans le dépôt ou dans une base, ni qui compile ou exécute : `scripts/*` (dont
`scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/
`switch`/`stash`/`reset`/`rebase`, `sqlx`, `cargo` (aucune sous-commande), `npm`, `npx`, `node`, `docker`, `gh` en écriture,
aucune requête SQL ni HTTP. Autorisés : lecture, `grep`, `sed -n`, `wc`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext`
vers `/home/gcorbaz/devel/kesh-gate-logs/`.
