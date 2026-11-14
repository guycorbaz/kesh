# Epic 36 — registre des choix autonomes de la story 36-3 (clé `36-3`)

Choix faits à la rédaction de la fiche `36-3-ecriture-montant-ambigu.md` (2026-11-14, `4de0bcc7e`), hors de ce que
C-e360-6 (amendé par Guy le 2026-11-14) tranche déjà. Chaque choix : contexte, option retenue, options écartées,
réversibilité.

### C-36-3-1 — Un lecteur facultatif pour `computeBalance` et `classifyLine`

- **Contexte** : `computeBalance` sert aussi les soldes de départ (`settings/opening-balances/+page.svelte:135`), hors
  périmètre (#1050) ; le journal doit admettre l'apostrophe et refuser l'ambiguïté.
- **Retenu** : un paramètre `AmountReader` facultatif ; le défaut (`defaultAmountReader`) est la lecture exacte
  d'aujourd'hui ; le journal passe `journalAmountReader` (fiche, D3).
- **Écartées** : (a) changer la lecture par défaut — change les soldes de départ sans le dire ; (b) une seconde paire de
  fonctions d'équilibre — duplication.
- **Réversible** : oui ; #1050 pourra passer le lecteur du journal (ou le sien) aux soldes de départ.

### C-36-3-2 — Groupes de milliers stricts, deux apostrophes, aucun blanc

- **Contexte** : Guy admet « l'apostrophe des milliers » au journal ; la 35-6 admet blancs et groupes libres.
- **Retenu** : apostrophes droite (U+0027) et typographique (U+2019, celle que Kesh affiche) ; groupes de trois chiffres
  exactement (`\d{1,3}(['’]\d{3})+`) ; pas de blanc ; pas de rognage (le journal n'en fait pas).
- **Écartées** : (a) la grammaire de la 35-6 — « 12'34 » n'a pas de lecture sûre, et les blancs ne sont pas demandés ;
  (b) l'apostrophe droite seule — un montant copié depuis un écran de Kesh (U+2019) serait refusé.
- **Réversible** : oui ; refus à l'écran seulement.

### C-36-3-3 — Le message « Maximum 4 décimales » remplacé par un message « illisible »

- **Contexte** : la clé `journal-entry-form-max-decimals` s'affiche pour tout montant invalide (« abc » compris) ; la
  grammaire neuve crée des formes invalides (« 12'34 ») qu'elle décrirait faussement.
- **Retenu** : clé retirée ; `journal-entry-form-amount-unreadable` qui décrit la forme admise ; clé
  `journal-entry-form-amount-ambiguous` pour l'ambiguïté, prioritaire sur la ligne. L'E2E `journal-entries.spec.ts:257`
  suit.
- **Écartées** : garder l'ancien message ; changer son seul texte (le nom de la clé mentirait).
- **Réversible** : oui.

### C-36-3-4 — L'assistant réemploie la clé du journal et passe une charge normalisée

- **Contexte** : l'assistant (même module) arrondit le HT avant de produire les lignes : l'ambiguïté disparaît à
  l'insertion ; il n'a pas de suite de composant.
- **Retenu** : `readHtAmount` pure dans `vat-purchase.ts` ; `insert()` passe la charge normalisée ; message
  `journal-entry-form-amount-ambiguous` réemployé (une entrée de lint de plus) ; autres refus : bordure seule, inchangé.
- **Écartées** : une clé `vat-purchase-*` au même texte ; une suite de composant neuve.
- **Réversible** : oui.

### C-36-3-5 — Les sites du même symptôme hors périmètre : une issue par site

- **Contexte** : l'inventaire par le symptôme (fiche, tableau) trouve, outre #1050 et #1159, sept sites sans issue :
  montant de paiement attendu d'une facture fournisseur, prix unitaire de la facture client, prix d'un article,
  montant d'un règlement de facture, frais de rappel, montant minimal de facture, filtres de montant de la liste des
  écritures.
- **Retenu** : hors périmètre (C-e360-6 amendé : « les autres sites ne s'intègrent pas ») ; une issue par site,
  demandée à l'orchestrateur ; le prix unitaire de la facture client signalé à part, sur le chemin de la première
  facture de Guy.
- **Écartée** : en intégrer un — contraire à la décision de Guy, et au-delà de cinq modules.
- **Réversible** : c'est à Guy d'en ranger un dans une story.

### C-36-3-6 — Le PDF du manuel se régénère à l'intégration

- **Contexte** : le fichier d'epic prévoit, pour `user-manual.tex` partagé, « PDF régénéré après l'union ».
- **Retenu** : la branche change le `.tex` ; le PDF et son contrôle aplati se font à l'intégration (AC10).
- **Écartée** : régénérer le PDF dans la branche — conflit binaire certain avec 36-4 et 36-5.
- **Réversible** : oui.

## Choix du développement (2026-11-14, étape 0 et suivantes)

### C-36-3-7 — Un premier groupe à zéro de tête est illisible (V1-2)

- **Contexte** : la grammaire de D2 admettait « 0'123.456 » (groupé, partie entière `0123`) : claire, charge
  `0123.456` ; rien n'en disait le sort (validation P1, V1-2).
- **Retenu** : le premier groupe d'une partie entière groupée commence par un chiffre non nul
  (`[1-9]\d{0,2}(?:['’]\d{3})+`) : « 0'123.456 », « 01'234 » → `unreadable`. Conséquence : dans la grammaire du journal,
  toute saisie groupée a quatre chiffres entiers ou plus sans zéro de tête, donc `!grouped` n'y change jamais le verdict ;
  la mutation M3 n'est tuée qu'au niveau du prédicat (AC1, cas `{ '1', '234', true }`) et par la 35-6 si sa table a un
  cas groupé court.
- **Écartées** : (a) la laisser claire — une saisie « 0'123 » n'a pas de sens, et la charge à zéro de tête est
  inutilement surprenante ; (b) la juger ambiguë — elle ne l'est pas.
- **Réversible** : oui ; refus à l'écran seulement.

### C-36-3-8 — Quinze chiffres entiers : une garde séparée (V1-1)

- **Contexte** : la regex groupée de D2 n'a pas de borne haute ; AC3 exige « 1'234'567'890'123'456 » → `unreadable`.
- **Retenu** : `JOURNAL_MAX_INTEGER_DIGITS = 15`, contrôlé sur les chiffres entiers une fois les apostrophes retirées.
- **Réversible** : oui.

### C-36-3-9 — L'écriture de l'assistant tirée d'une fonction pure (V1-7)

- **Contexte** : `insert()` de `VatPurchaseAssistant.svelte` lisait le HT brut ; la mutation M8 (« HT brut passé à
  `buildPurchaseVatLines` ») n'était tuée que par l'E2E, qui ne tourne qu'à la clôture.
- **Retenu** : `purchaseVatEntry(params)` dans `vat-purchase.ts` lit le HT brut par `readHtAmount`, rend `null` s'il
  n'est pas utilisable, sinon les lignes sur la charge normalisée et `isExempt` ; `insert()` n'appelle qu'elle. M8 se
  joue sur cette fonction et `vat-purchase.test.ts` la tue.
- **Écartée** : une suite de composant pour l'assistant (D5, écartée par la fiche).
- **Réversible** : oui.

### C-36-3-10 — Deux fonctions d'aide dans `balance.ts` pour l'écran

- **Contexte** : D4 décrit le message de ligne (ambigu prioritaire) et la charge normalisée, sans dire où vit la logique.
- **Retenu** : `lineAmountProblem(debit, credit)` et `journalAmountPayload(raw)` (`'0'` pour le vide, `null` pour un
  refus), testées dans `balance.test.ts` ; le formulaire n'envoie rien si une charge est `null` (garde redondante avec
  `canSubmit`, plutôt que de deviner un montant).
- **Écartée** : la logique en ligne dans le gabarit — non testable sans composant (M10).
- **Réversible** : oui.

### C-36-3-11 — Le manuel dit l'espace refusée et borne le renvoi à la 35-6 (V1-8)

- **Contexte** : le texte proposé par AC10 renvoyait au solde d'ouverture, dont la grammaire admet blancs et souligné.
- **Retenu** : « une espace entre les milliers n'est pas admise » ; « la même règle d'ambiguïté que pour le solde
  d'ouverture » ; une phrase sur « Montant illisible ».
- **Réversible** : oui.
