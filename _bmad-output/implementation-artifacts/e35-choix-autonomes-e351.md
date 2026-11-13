# Epic 35 — choix autonomes de la story 35-1 (clé `e351`, #932, #948)

Registre tenu par l'agent de développement. Mise à jour par union, jamais par écrasement.

### C-e351-1 — Au serveur, un format inchangé n'est dispensé que du compteur

- **Contexte** : D2 (C-e350-43) dit « la route valide le format client et le format d'avoir quand la requête le porte et
  qu'il diffère de la valeur enregistrée », et, pour l'écran, que seul `NoSeq` est toléré sur une valeur inchangée, les
  autres refus restant inconditionnels.
- **Retenu** : la même règle des deux côtés — `validate_template_change(nouveau, enregistré)` (`kesh-core`) rend
  `validate_template`, sauf `NoSeq` quand `nouveau == enregistré`. Un format enregistré invalide pour une autre raison
  (posé par SQL) bloque comme avant la story.
- **Écartée** : ne rien juger d'un format inchangé — plus large que l'écran, et un format illisible enregistré hors de
  l'application passerait sans un mot à chaque enregistrement.
- **Réversible** : oui (une fonction, aucune donnée écrite).

### C-e351-2 — Un nom d'exercice à accolade ne bloque le rendu que sous `{FY}`

- **Contexte** : à l'étape 0, `render` (`invoice_format.rs`, avant la story : `if fy_name.contains('{') ||
  fy_name.contains('}') { return Err(FormatError::IllegalCharacter(…)) }`, placé **avant** l'analyse du format) refusait
  un nom d'exercice à accolade **quel que soit le format** ; or `PUT /fiscal-years/{id}` l'enregistre (constaté :
  `rouges.log`, AC3, 200 au renommage « Ex {2026} »). Un exercice ainsi nommé bloquait donc toute validation, au format
  par défaut `F-{YEAR}-{SEQ:04}` compris, en 500.
- **Retenu** : le refus n'a lieu que si le format reprend le nom (`Token::Fy`), sous une variante propre
  (`FormatError::FiscalYearNameBrace`) que la conversion de D4 nomme `brace_in_fiscal_year_name`. Limite voisine courte,
  dans la zone (même fonction, même défaut 500).
- **Écartées** : garder le refus inconditionnel (un 400 au lieu d'un 500, mais une validation bloquée par un nom qui
  n'entre pas dans le numéro) ; refuser l'accolade au renommage de l'exercice (hors périmètre, route des exercices —
  demandé en issue si l'orchestrateur le juge utile).
- **Réversible** : oui (code seul).

### C-e351-3 — Les valeurs de `details` : `invoice`, `credit_note`, et les quatre causes en minuscules soulignées

- **Contexte** : D4 écrit les causes `too_long`, `brace_in_fiscal_year_name`, `duplicate`, `invalid_format` et
  `document: "invoice"` ; la valeur de l'avoir n'est pas écrite.
- **Retenu** : `credit_note`, dans la graphie des causes (précédent : `details.reason = "counterparty_equals_bank_ledger"`).
- **Écartée** : `creditNote` (graphie des clés JSON, non des valeurs ; mélangerait deux graphies dans un même objet).
- **Réversible** : tant que la 0.17.7 n'est pas publiée ; ensuite, contrat d'API.

### C-e351-4 — `NoSeq` en dernier, et hors du format du numéro interne

- **Contexte** : D1 place `NoSeq` après `NoPlaceholder` ; D2 (V4-1) écrit la branche du miroir « dans
  `validateTemplateRules`, après la boucle des placeholders ». AC1 exige `validate_numbering_template` inchangée, ses
  tests verts — or elle appelait `validate_template` : `FF-{YEAR}` y aurait rendu `NoSeq` au lieu de `NotNumberingShape`.
- **Retenu** : `NoSeq` est testé **en dernier**, après le rendu au pire (un `NO_SEQ` dit que toutes les autres règles
  sont tenues, ce qui rend sûre la tolérance de D2 : un format inchangé trop long et sans compteur reste bloqué). Les
  règles communes passent dans `validate_template_rules` (privée) ; `validate_template` = règles + compteur ;
  `validate_numbering_template` = règles + forme, comme avant. Le miroir suit la même structure : la branche `NO_SEQ`
  est dans `validateFormatTemplate`, pour les seuls formats client et avoir (`numbering` absent), après
  `validateTemplateRules` — non dans `validateTemplateRules`, où elle aurait changé le message du numéro interne.
- **Écartée** : la lettre de V4-1 (branche dans `validateTemplateRules`) — elle aurait fait dire « compteur » au refus
  de forme du numéro interne et cassé la parité avec le Rust.
- **Réversible** : oui.

### C-e351-5 — Le message du refus de D4 : une phrase sur la pièce, une sur la cause

- **Contexte** : D4 demande un message i18n ×4 « qui nomme le format, l'exercice et la cause, et renvoie à
  *Administration → Facturation* (ou au nom de l'exercice) » ; quatre causes × deux pièces.
- **Retenu** : six clés `error-number-format-unrenderable-*` — deux pour la pièce (« Cette facture ne peut pas être
  validée… », « Cet avoir ne peut pas être créé… », qui disent aussi que rien n'est enregistré), quatre pour la cause,
  paramétrées par `{ $format }` et `{ $fiscalYear }`. Aucun sélecteur Fluent.
- **Écartée** : huit clés (une par pièce et par cause) ; une seule clé à sélecteur sur `$document` (inscription à
  `SELECTEURS_RESOLUS_COTE_SERVEUR`, sans gain pour le lecteur).
- **Réversible** : oui (catalogue).

### C-e351-6 — L'aide non bloquante reprend le texte du refus, en ambre

- **Contexte** : D2 prescrit, sous un format chargé sans compteur et inchangé, « une aide non bloquante (la clé de D1, en
  texte d'avertissement, non destructif) ».
- **Retenu** : le texte de `invoices-format-error-no-seq`, en `text-amber-800` (la couleur des avertissements de la page,
  `border-amber-400 bg-amber-50 … text-amber-900` pour l'accès réservé), sous l'aperçu ; testid
  `settings-format-noseq-hint` et `settings-credit-note-format-noseq-hint`.
- **Écartée** : une clé neuve « ce format enregistré n'a pas de compteur » — plus juste dans le ton, mais la fiche la
  désigne ; à reprendre si la recette de Guy trouve le texte trop impératif.
- **Réversible** : oui.
