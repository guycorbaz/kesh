# Epic 35 — registre des choix autonomes du kickoff et des fiches 35-2, 35-3 (clé `e350`)

Registre propre au kickoff (story 35-0), **versé par union** par l'orchestrateur. Chaque choix : contexte, option
retenue, options écartées, réversibilité. Lignes citées sur `release/0.17.6` (`4e1c768a`).

---

### C-e350-1 — #944 n'entre pas dans la 35-2

- **Contexte** : le mandat laisse #944 « en option à trancher » pour la story du refus du PDF. Lu à l'issue, #944 n'est
  pas un point mais huit : ligne verticale, ciseaux ou « Separate before paying in » (`crates/kesh-qrbill/src/pdf.rs:728-744`,
  « Scissors glyph … omitted in v0.1 »), place de *Payable by*, largeur du message, très longues adresses, coupe de ligne
  calée sur les capitales (27 `W` débordent sur le QR), *Acceptance point* recouvert, cadre vide du débiteur absent,
  type K. Impact I3 : le code QR se lit, la banque accepte.
- **Retenu** : #944 hors de la 35-2, laissé ouvert, proposé à l'Epic 36 comme story à lui seul.
- **Écartées** : (a) #944 entier dans la 35-2 : autre crate (`kesh-qrbill`), golden tests et captures du PDF, boucle
  complète ; la story qui doit sortir avant le 2026-11-30 deviendrait la plus lourde de l'epic ; (b) la seule ligne
  verticale avec le texte « Separate before paying in » : découpe l'issue en laissant ses autres points, et touche
  quand même le rendu de chaque QR-facture à une semaine de la première facture réelle.
- **Réversible** : oui ; une story 35-9 peut le prendre si l'epic finit tôt.

### C-e350-2 — #537 : le type se revérifie à l'usage, pas au retypage

- **Contexte** : #537 propose deux remèdes, refuser ou signaler le retypage d'un compte désigné, ou revérifier le type à
  l'usage. Lu au code : `lock_designated_accounts_in_tx` lit `id, number, active, postable` sans `account_type`
  (`company_invoice_settings.rs:960-970`), et son doc-comment écrit l'angle mort (`:902-904`). Le retypage passe par
  `accounts::update` (`repositories/accounts.rs:622`), dont `check_role_account_type` ne contrôle que la paire
  rôle/type **demandée** : retirer le rôle et changer le type dans la même requête passe.
- **Retenu** : revérifier le type dans `DesignatedAccountsSnapshot::check_written`, pour les quatre rôles désignés
  (créance et TVA due à la validation d'une facture ; créanciers et TVA récupérable à celle d'une facture fournisseur),
  avec un refus nommé, typé et traduit. Le commentaire de #537 (« même forme côté dette ») est ainsi traité avec elle.
- **Écartées** : (a) refuser le retypage d'un compte désigné : une seconde garde, sur un autre chemin, qui laisse
  passer tout autre moyen de changer le type ; (b) avertir au retypage : utile, mais ce n'est pas la garde ; à proposer
  en issue si la story le juge utile ; (c) l'avoir : hors de la story, comme le dit le doc-comment (C35 de la 15-5d) —
  une facture émise doit rester annulable.
- **Réversible** : oui ; un refus en plus, aucune donnée écrite.

### C-e350-3 — #955 : le serveur nomme le champ, l'écran reprend son message

- **Contexte** : `INVOICE_NOT_PDF_READY` est rendu sans `details` (`errors.rs:2073-2075`, `build_response` à `:1420`) ;
  le message vient de `map_qrbill_error` (`invoice_pdf_service.rs:494-541`), en français en dur et avec des noms
  techniques (`debtor.name`). L'écran de la fiche le remplace par une clé fixe (`pdf-error.ts:18`, `:44-45`) ; le lot de
  rappels n'affiche que le code (`ReminderBatchReport.svelte:30`) ; l'envoi par e-mail affiche le message brut
  (`invoices/[id]/+page.svelte:866`).
- **Retenu** : le refus porte des `details` structurés (champ, raison, caractère et son point de code) et un message
  **résolu au serveur** dans la langue de l'installation, qui nomme le champ en mots de l'utilisateur (« la localité du
  client ») ; l'écran reprend ce message quand les `details` sont là, comme il le fait déjà pour les codes de
  `SERVER_RESOLVED_CODES` (`pdf-error.ts:36`). Le lot de rappels montre la même phrase. Forme exacte : la fiche 35-2.
- **Écartées** : (a) afficher le message brut du serveur tel qu'il est : technique et non traduit ; (b) résoudre au
  frontend à partir des `details` seuls : deux tables de libellés de champ à tenir (serveur et écran) pour un même
  message.
- **Réversible** : oui ; `details` s'ajoute au contrat sans rien retirer.

### C-e350-4 — 35-2 et 35-3 en parallèle malgré trois fichiers communs

- **Contexte** : les deux doivent sortir avant le 2026-11-30 ; elles touchent `crates/kesh-api/src/errors.rs`, les
  quatre `messages.ftl` et `user-manual.tex`, à des passages distincts.
- **Retenu** : parallèle, union à l'intégration, PDF du manuel régénéré une fois après l'union.
- **Écartée** : en série (35-3 puis 35-2) : sûr, mais retarde la seconde d'une boucle complète.
- **Réversible** : oui.

### C-e350-5 — La règle d'arrêt vaut pour toute migration

- **Contexte** : le mandat dit « migration qui empêche le retour à 0.17.6 → s'arrêter ». Le `CLAUDE.md` (§ *Migration
  breaking policy*, P1, constaté à la release 0.13.3) établit que toute migration empêche le binaire précédent de
  démarrer.
- **Retenu** : toute migration déclenche l'arrêt de l'orchestrateur ; une fiche qui en envisage une le dit avant de
  l'écrire.
- **Écartée** : ne s'arrêter que sur une migration *breaking* (`kesh_version_min_required` relevé) : contredit le fait
  constaté, le retour à la 0.17.6 passerait alors par la sauvegarde.
- **Réversible** : c'est à Guy de l'assouplir.

### C-e350-6 — Ordre après la tête

- **Retenu** : 35-4, 35-6, 35-5 (I1), puis 35-7, 35-8, 35-1 (I2), 35-1 en dernier (format par défaut gardé, probabilité
  basse), 35-8 après 35-5 et 35-3 (fichiers communs).
- **Réversible** : oui.
