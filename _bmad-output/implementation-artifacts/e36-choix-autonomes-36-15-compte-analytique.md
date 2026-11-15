# Story 36-15 — choix autonomes (`C-36-15-k`)

Registre de la story `36-15-compte-analytique` (#1161). Mise à jour par union, jamais par écrasement. Les choix du
kickoff (C-e360-15) font foi ; ceux-ci les complètent.

### C-36-15-1 — Les traductions de « compte analytique » (fiche, D2) — **en attente de l'arbitrage de Guy**

- **Contexte** : le CR #1161 demande que la fiche fixe, une fois, les termes de-CH, it-CH et en ; il propose
  « Kostenstelle » ou « Kostenträger », « centro di costo » ou « conto analitico », « cost centre » ou « analytical
  account ». Le glossaire (`docs/i18n-glossaire.md:172-189`) porte l'arbitrage de Guy du 2026-08-19 (« on ne traduit pas
  « analytique » », `Kostenstelle` écarté).
- **Acquis** : fr-CH « compte analytique » (CR, recommandation 2).
- **En attente** (validation P1, V1-3 ; page de décision de l'orchestrateur) : **A** analytisches Konto / conto analitico
  / analytical account — recommandée par l'orchestrateur, suit le français (glossaire `:29`) ; **B** Kostenstelle /
  centro di costo / cost centre ; **C** Kostenstelle / conto analitico / analytical account (première version de la fiche,
  `54b513a71`). La fiche ne tranche pas ; la table clé par clé est donnée pour chaque terme.
- **Fondement** : le CR #1161 de Guy, non un changement de l'objet (la table `projects` est la même qu'au 2026-08-19).
- **Écartées d'emblée** : `Kostenträger`, `commessa` (trop étroits) ; garder `Projekt` / `progetto` / *project*.
- **À écrire à la réponse** : date, option, terme par langue, ici et au Change Log de la fiche ; le glossaire suit (D2, fin).
- **Réversible** : oui, textes seulement (49 valeurs par langue).
- *Annoté le 2026-11-15 (remédiation P1)* : la première rédaction retenait l'option C et la disait « signalée au
  rapport » ; elle n'était pas à l'agent de trancher.

### C-36-15-2 — Les refus de `/api/v1/projects` changent de mot (fiche, D4)

- **Contexte** : onze `AppError::Validation` de `routes/projects.rs` sont affichés tels quels par l'écran
  (`settings/projects/+page.svelte:124-125`, `:144`).
- **Retenu** : ils disent « compte analytique » ; codes et statuts inchangés ; la note de version le signale.
- **Écartées** : les laisser à #710 (l'écran dirait « projet » sous un titre « Comptes analytiques ») ; les traduire ici
  (hors périmètre, #710).
- **Réversible** : oui.

### C-36-15-3 — Les en-têtes CSV `Projet;SousProjet` restent (fiche, D5)

- **Contexte** : `kesh-report/src/csv.rs:547-548`, `:616-617` ; noms de colonnes en forme d'identifiant, lus par des
  tableurs et des macros ; affirmés par `project_reports_e2e.rs:218`, `:350`.
- **Retenu** : inchangés, comme les identifiants de l'API.
- **Écartée** : `CompteAnalytique;SousCompteAnalytique`.
- **Réversible** : oui (deux lignes et deux assertions) ; à demander à Guy s'il le souhaite.

### C-36-15-4 — Le `<th>Projet</th>` en dur du détail d'écriture change de texte, reste en dur (fiche, D7)

- **Contexte** : `routes/(app)/journal-entries/[id]/+page.svelte:611`, voisin de `:609` (`Compte`), lui aussi en dur.
- **Retenu** : « Compte analytique », littéral ; la mise sous `i18nMsg` reste à #339.
- **Écartée** : une clé neuve ×4 (change `sitesTotal`, hors vocabulaire).
- **Réversible** : oui.

### C-36-15-5 — Les PDF des manuels se régénèrent à l'intégration (fiche, D8)

- **Contexte** : `user-manual.tex` est commun à d'autres stories de la 0.18.0 ; un PDF binaire ne s'unit pas.
- **Retenu** : le `.tex` dans la story ; compilation locale pour l'AC-6, PDF rendus à leur état ; régénération après
  l'union, contrôlée aplatie (E6).
- **Écartée** : commiter les PDF dans la story (conflit binaire à chaque union).
- **Réversible** : oui.

### C-36-15-6 — Deux gardes par le symptôme plutôt qu'une liste de clés (fiche, AC-7, AC-8)

- **Contexte** : un renommage à moitié fait est le défaut que le CR redoute ; une liste de 49 clés ne verrait pas un
  libellé en dur ni un repli oublié.
- **Retenu** : G25 dans `crates/kesh-api/tests/textes_coherents.rs` (catalogues, manuels, `api-external.md`, README,
  `projects.rs`, `pdf.rs`) et `frontend/src/lib/shared/vocabulaire-analytique.test.ts` (sources du frontend, commentaires
  masqués, repli = catalogue) ; listes d'exceptions fermées, chacune devant encore se rencontrer ; sondes positives.
- **Écartée** : une seule garde Rust lisant aussi le Svelte (le masquage des commentaires existe en JS, pas en Rust).
- **Réversible** : oui.
- *Annoté le 2026-11-15 (remédiation P1, V1-1, V1-2, V1-4, V1-5)* : côté Rust, G25 ne lit que les littéraux de chaîne
  (extracteur testé) ; côté catalogues, la valeur seule, casse ignorée ; exceptions par extrait de texte, non par numéro
  de ligne ; la garde Vitest lit aussi les replis en objet et celui du nom de fichier.
