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

- **Retenu** : un seul `UPDATE … COALESCE …, version = version + 1` dans la transaction de la finalisation, seulement si
  une colonne change ; audit `company_invoice_settings.updated` avec `trigger: "onboarding"`, `before`, `after`.
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
- **Retenu** : un compte débiteurs ou créanciers tiré du **rôle** et lié à un compte bancaire non archivé fait refuser
  la finalisation, avec un motif qui nomme le compte bancaire ; `refs #987` (la partie « désignation de rôle » y reste).
- **Écartés** : laisser le créancier vide en silence ; `400 BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT`.
- **Réversible** : oui (délier le compte bancaire, ou désigner un autre compte, puis réessayer).
- **À trancher par l'orchestrateur** : garder #987 ouverte en `refs`, ou la scinder.

### C-36-19-6 — L'E2E n'affirme rien de l'écran d'après la finalisation (fiche, AC8)

- **Contexte** : la 36-17 change cet écran ; les deux stories ne doivent pas dépendre l'une de l'autre.
- **Retenu** : spec neuf, l'action par l'interface (journal ouvert, puis *Configurer plus tard*), le résultat lu par
  l'API (`stepCompleted`, réglages).
- **Écarté** : attendre l'URL `/` ou la page « Configuration terminée ».
- **Réversible** : oui.
