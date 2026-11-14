# Epic 35 — choix autonomes de la story 35-3 (clé `e353`, #537)

Registre tenu par l'agent de développement. Mise à jour par union, jamais par écrasement.

### C-e353-1 — Le code du refus : `DESIGNATED_ACCOUNT_WRONG_TYPE`

- **Contexte** : D2 de la fiche propose ce nom « à confirmer au registre `e353` » ; AC11 et AC13 l'attendent.
- **Retenu** : `DESIGNATED_ACCOUNT_WRONG_TYPE` (400), variante `DbError::DesignatedAccountsWrongType`, détail
  `details.rejected[{accountId, accountNumber, setting, expectedType, actualType}]`. Le singulier suit
  `ACCOUNT_NOT_POSTABLE`, qui nomme aussi plusieurs comptes.
- **Écartées** : `DESIGNATED_ACCOUNTS_WRONG_TYPE` (pluriel, sans précédent parmi les codes) ; `ACCOUNT_WRONG_TYPE` (ne dit
  pas que le compte vient des réglages).
- **Réversible** : tant que la 0.17.7 n'est pas publiée ; ensuite, c'est un contrat d'API.

### C-e353-2 — Les libellés des réglages reprennent ceux de la page

- **Contexte** : D2 propose « compte débiteurs », « compte de TVA due », « compte de TVA récupérable » ; la page
  *Administration → Facturation* titre ses champs « Compte créance client (Actif) », « Compte TVA due (Passif) »,
  « Compte créanciers (Passif) », « Compte TVA récupérable (Actif) » (`settings-invoicing-receivable-account`,
  `invoices-settings-vat-payable`…). Le message envoie l'administrateur sur cette page.
- **Retenu** : le libellé du champ, en minuscule et sans le type entre parenthèses — « compte créance client »,
  « compte TVA due », « compte créanciers », « compte TVA récupérable » ; dans les trois autres langues, le libellé de la
  page de la même façon. L'anglais porte l'article dans les libellés de type (« an asset account », « a liability
  account ») : la phrase n'a pas d'article à accorder.
- **Écartée** : les libellés de D2 tels quels — l'administrateur chercherait un champ « débiteurs » que la page n'a pas.
- **Réversible** : oui (catalogue et replis).

### C-e353-3 — L'avertissement au retypage reste à #1047

- **Contexte** : D1 (C-e350-2) garde le type à l'usage et ne refuse pas le retypage ; tâche V3-6.
- **Retenu** : rien à l'écran du plan comptable dans cette story ; l'avertissement au retypage d'un compte désigné est
  tracé par **#1047** (précédent : `accounts::archive_guarded`, `accounts.rs:859`, et
  `company_invoice_settings::designations_of_account`, `:62`). Le manuel utilisateur dit, en attendant, qu'un compte
  désigné ne se retype pas sans reprendre la désignation.
- **Réversible** : oui.
