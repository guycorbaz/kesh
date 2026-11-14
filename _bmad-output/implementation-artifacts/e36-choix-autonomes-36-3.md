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
