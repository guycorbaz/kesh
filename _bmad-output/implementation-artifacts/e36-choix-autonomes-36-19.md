# Epic 36 — registre des choix autonomes de la story 36-19 (clé `36-19`)

Registre propre à la story 36-19 (#1200), **versé par union**. Chaque choix : contexte, option retenue, options écartées,
réversibilité. Code lu à `fed34a9c7`. Fiche : `36-19-finalisation-comptes-vides.md` (décisions D1 à D7). Les choix du
kickoff (`e36-choix-autonomes-e360.md`, C-e360-3 sans migration, C-e360-11 note de version) font foi.

---

### C-36-19-1 — Ce que la finalisation complète (fiche, D2)

- **Contexte** : sans migration, rien ne distingue un réglage « jamais rempli » d'un réglage « vidé par l'utilisateur »,
  sauf la `version` (seul un `PUT` qui change quelque chose l'incrémente) et l'état des dix colonnes de compte.
- **Retenu** : débiteurs et produit vides → toujours complétés par leur rôle ; les cinq facultatifs → complétés
  seulement sur une ligne **vierge** (`version = 1`, dix comptes vides) ; aucune valeur non vide remplacée ; TVA jamais.
- **Écartés** : tout compléter toujours ; ne compléter que les deux obligatoires ; marqueur de ligne (migration).
- **Réversible** : oui — un compte complété se change ou se vide dans *Administration → Facturation*.

### C-36-19-2 — Le rôle n'est cherché que pour un réglage vide (D3)

- **Retenu** : un débiteurs ou un produit désigné et actif suffit, même sans compte de rôle ; désigné mais archivé →
  refus nommé, jamais remplacé.
- **Écarté** : garder le refus anticipé quand le rôle manque.
- **Réversible** : oui.

### C-36-19-3 — Complément atomique, `version` incrémentée, audit `…updated` (D4)

- **Retenu** : un seul `UPDATE … COALESCE …` dans la transaction de la finalisation, seulement si une colonne change,
  avec `version = version + 1` **seulement si la ligne existait** (C-36-19-7 ; aligné à la remédiation de la validation
  P2, V2-5) ; audit `company_invoice_settings.updated` avec `trigger: "onboarding"`, `before`, `after`.
- **Écartés** : pas de `version` (un écran ouvert avant effacerait le complément) ; action d'audit neuve.
- **Réversible** : oui (aucun schéma, aucune donnée réécrite hors de la finalisation).

### C-36-19-4 — Le motif : `400 VALIDATION_ERROR`, message qui nomme la cause (D5)

- **Retenu** : code et statut inchangés (la 36-17 reconnaît le refus au code) ; message français au gabarit de la
  fiche ; variante de `DbError` rendue par une fonction neuve, ramenée à `InactiveOrInvalidAccounts` par l'enveloppe
  `insert_with_defaults_in_tx` (démonstration, jeux de pièges et tests existants inchangés ; `kesh-seed` non touché).
- **Écartés** : remplacer la variante partout ; diagnostic relu après coup ; code d'erreur neuf.
- **Réversible** : oui.

### C-36-19-5 — Partie « finalisation » de #987 absorbée (D6)

- **Contexte** : compléter la ligne ouvrirait un chemin aujourd'hui fermé par le refus : compte bancaire lié au compte
  1100 aux étapes 6 et 7, puis 1100 désigné compte débiteurs (état interdit par la 15-6c).
- **Retenu** *(récrit à la remédiation de la validation P1, V1-2, sur arbitrage de l'orchestrateur, choix prudent)* : un
  candidat invalide n'est jamais désigné. Pour les deux **obligatoires** (débiteurs, produit), il fait refuser la
  finalisation, avec un motif qui nomme le compte bancaire et le remède ; pour un **facultatif** (créanciers, arrondi,
  escompte, frais, pertes), le réglage **reste vide, sans refuser**. `refs #987` (la partie « désignation de rôle » y
  reste). *Étendu à la remédiation de la validation P2 (V2-2, arbitrage de l'orchestrateur)* : un compte de rôle **non
  imputable** est aussi un candidat invalide (filtre posé dans la fonction neuve, non dans `find_singleton_role_in_tx`).
- **Écartés** : refuser aussi pour un créanciers lié (lettre de #987 ; bloquerait l'assistant pour un réglage
  facultatif — c'était le choix de la première rédaction) ; `400 BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT`.
- **Réversible** : oui (délier le compte bancaire, ou désigner un autre compte, puis réessayer ; un créanciers vide se
  désigne dans *Administration → Facturation*).
- **À trancher par l'orchestrateur** : garder #987 ouverte en `refs`, ou la scinder.

### C-36-19-6 — L'E2E n'affirme rien de l'écran d'après la finalisation (fiche, AC8)

- **Contexte** : la 36-17 change cet écran ; les deux stories ne doivent pas dépendre l'une de l'autre.
- **Retenu** : spec neuf, l'action par l'interface (journal ouvert, puis *Configurer plus tard*), le résultat lu par
  l'API (`stepCompleted`, réglages).
- **Écarté** : attendre l'URL `/` ou la page « Configuration terminée ».
- **Réversible** : oui.

### C-36-19-7 — Un seul ordre pour une ligne neuve et une ligne existante (fiche, D4 ; validation P1, V1-3)

- **Contexte** : les contrôles de D6 lisent `bank_accounts` en partagé ; le geste bancaire prend les réglages avant les
  comptes bancaires. Un contrôle placé avant l'insertion d'une ligne neuve inverserait cet ordre.
- **Retenu** : comptes candidats, puis `INSERT IGNORE` nu, `FOR UPDATE` de la ligne, contrôles, et un seul `UPDATE …
  COALESCE` ; `version` incrémentée seulement si la ligne existait (une ligne neuve garde `version` 1, comme aujourd'hui).
- **Écartés** : contrôler avant l'insertion ; garder deux chemins (insertion complète, complément) aux ordres distincts.
- **Réversible** : oui.

### C-36-19-8 — Bras de rendu de la variante traduit (fiche, D5 ; validation P1, V1-6)

- **Contexte** : le rendu des `DbError` passe par `t(clé, repli)` ; la finalisation convertit elle-même le refus, si
  bien que le bras n'est atteint par aucune route, mais le match l'exige.
- **Retenu** : clé neuve `error-default-invoice-accounts-unavailable` dans les quatre locales, repli égal à la valeur
  `fr-CH`, `400 VALIDATION_ERROR` ; `kesh-i18n` devient le cinquième module. Le refus de `seed_demo` (`onboarding.rs:271`)
  reste hors périmètre (cause « réglage vide » non atteignable par la démonstration).
- **Écartés** : un repli sans clé (rendu non traduit, à côté de bras traduits) ; un bras `unreachable!()` (interdit par le
  `CLAUDE.md`, § *Pattern batch*, garde-fou défensif).
- **Réversible** : oui.

### C-36-19-9 — La lecture de `postable` du candidat : une seconde lecture verrouillante (développement, V3-1)

- **Contexte** : V3-1 exige une lecture verrouillante ; deux formes possibles — étendre la requête de rôle, ou relire la
  ligne déjà verrouillée.
- **Retenu** : `find_singleton_role_in_tx` reste la source unique de la recherche du rôle (inchangée, partagée avec
  l'étape 6) ; la fonction neuve relit ensuite `number, postable` du candidat par `SELECT … WHERE id = ? AND
  company_id = ? FOR UPDATE` — une ligne que la transaction tient déjà, donc aucun verrou neuf, et une lecture du dernier
  état validé.
- **Écartés** : une variante de `find_singleton_role_in_tx` qui rendrait `postable` (deux formes de la même recherche à
  tenir égales) ; une lecture simple (le cliché de V3-1).
- **Réversible** : oui.

### C-36-19-10 — Le compte obligatoire déjà désigné : contrôlé sous verrou partagé, après la ligne (développement, D3)

- **Contexte** : D3 refuse un débiteurs ou un produit désigné **archivé**. On ne sait quels comptes sont désignés
  qu'après avoir lu la ligne des réglages (étape 3 de D4) ; l'ancien code le jugeait par une jointure non verrouillante.
- **Retenu** : après le `FOR UPDATE` de la ligne, `SELECT number, active FROM accounts WHERE id = ? AND company_id = ?
  LOCK IN SHARE MODE` pour chacun des deux, débiteurs d'abord ; absent de la société ou archivé → cause « archivé »
  (numéro rendu s'il existe). Le verrou partagé est pris **après** la ligne des réglages, l'ordre de la validation d'une
  facture (réglages, puis comptes) ; quand le compte désigné est le compte de rôle — le cas courant —, la ligne est déjà
  tenue en exclusif et rien de neuf n'est pris. Un interblocage reste rejoué (`onboarding.rs:894`).
- **Écartés** : une lecture simple (cliché, comme V3-1) ; lire la ligne avant les comptes candidats (inverserait
  `accounts → company_invoice_settings`).
- **Réversible** : oui.

### C-36-19-11 — Forme de l'issue de la fonction neuve (développement, D4)

- **Retenu** : `company_invoice_settings::complete_default_accounts_in_tx` rend
  `DefaultAccountsOutcome::{Inserted(réglages), Completed { before, after }, Unchanged(réglages)}` ; la route écrit
  `…created` pour `Inserted`, `…updated` (`trigger` = `onboarding`, `before`, `after` au format de
  `settings_snapshot_json`) pour `Completed`, rien pour `Unchanged`.
- **Écartés** : un triplet de booléens ; écrire l'audit dans le dépôt (la route l'écrit déjà pour `…created`, par
  `from_current_user`).
- **Réversible** : oui.
