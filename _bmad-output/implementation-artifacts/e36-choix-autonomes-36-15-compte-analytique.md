# Story 36-15 — choix autonomes (`C-36-15-k`)

Registre de la story `36-15-compte-analytique` (#1161). Mise à jour par union, jamais par écrasement. Les choix du
kickoff (C-e360-15) font foi ; ceux-ci les complètent.

### C-36-15-1 — Les traductions de « compte analytique » (fiche, D2)

- **Contexte** : le CR #1161 demande que la fiche fixe, une fois, les termes de-CH, it-CH et en ; il propose
  « Kostenstelle » ou « Kostenträger », « centro di costo » ou « conto analitico », « cost centre » ou « analytical
  account ». Le glossaire (`docs/i18n-glossaire.md:172-189`) porte l'arbitrage de Guy du 2026-08-19 qui écartait
  `Kostenstelle` pour un objet de type *projet*.
- **Retenu** : **Kostenstelle** (f.), **conto analitico**, **analytical account** ; hiérarchie *übergeordnete
  Kostenstelle* / *Unterkostenstelle* / *Hauptkostenstelle*, *conto analitico superiore* / *sottoconto analitico* /
  *conto analitico radice*, *parent analytical account* / *analytical sub-account* / *root analytical account*.
- **Écartées** : `Kostenträger` (le « pour quoi ? » de 2026-08-19, trop étroit pour un immeuble ou un centre de coûts) ;
  `Analysekonto` (calque, collision avec `Konto`) ; `centro di costo` et *cost centre* (coûts seuls, alors que le
  rendement compte des revenus) ; garder `Projekt` / `progetto` / *project*.
- **Réversible** : oui, textes seulement (49 valeurs par langue). ⚠️ Défait un arbitrage de Guy, sur le CR de Guy qui
  le demande : signalé au rapport.

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
