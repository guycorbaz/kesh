# Epic 33 — registre des choix autonomes, stories 33-2, 33-3a, 33-3b (clé `e332`)

Registre propre à ces trois fiches, **versé par union** par l'orchestrateur. Chaque choix : contexte, option retenue, alternatives
écartées, réversibilité. Numéros de ligne de `4ef19901`.

---

## Story 33-2 — l'import CAMT : #922 et #915

### C-e332-1 — Le refus est inconditionnel

- **Contexte** : décision de Guy du 2026-11-06 : si la relecture échoue, refuser. Un fichier qu'accepte la 0.17.4 et dont la relecture
  échoue serait refusé même sans aucune ligne dans la fenêtre, ou sur une installation neuve.
- **Retenu** : refus dès qu'une des causes survient, sans considérer le contenu du compte, sans contournement par `confirm*`.
- **Écartées** : (a) refuser seulement si le compte a des lignes dans la fenêtre : le premier relevé passe, les suivants d'une même banque
  sont refusés, à cause de lignes que la 0.17.5 a elle-même lues, ce qui est incompréhensible ; (b) refuser seulement si une référence porte
  une entité : exigerait une troisième lecture du XML pour un cas qu'aucune banque connue ne produit ; (c) un `confirm` de plus : un
  contournement rouvre #922, et Guy a dit « refuse ».
- **Réversible** : oui, une condition à ajouter.

### C-e332-2 — Le désalignement est une cause de refus

- **Contexte** : #923 relève en LOW un alignement « fondé sur le seul nombre de lignes ». Lu au code, c'est plus : deux relevés du même IBAN,
  le premier à référence de caractère, font lire aux deux lectures deux relevés différents ; à nombre égal, le `zip` fabrique des alias sans
  rapport, sans aucun signal.
- **Retenu** : une paire dont la date de comptabilisation ou le montant normalisé diffère = échec de la relecture, `cause = "misaligned"`.
- **Écartées** : (a) comparer `statement_id` : lu tronqué en 0.17.3 s'il porte une entité, d'où un refus à tort ; (b) sélectionner par position
  dans le fichier : plus juste, mais change la signature de sélection pour une forme hypothétique, et une sélection qui diverge reste suspecte.
- **Réversible** : oui.

### C-e332-3 — Un alias ne reconnaît une ligne que si tous ses textes sont ceux de la lecture héritée

- **Contexte** : #915 laisse trois voies : garder, borner dans le temps, ne plus écarter par défaut. Une ligne importée par la 0.17.x porte en
  base **les textes** de cette lecture, jamais modifiés ensuite (17 sites `UPDATE`, aucun sur ces colonnes), et leur calcul n'a pas changé
  depuis l'Epic 8 hors #819 et la devise.
- **Retenu** : égalité stricte sur `value_date`, `reference`, `end_to_end_id`, `transaction_id`, `counterparty_iban`, `counterparty_name`,
  `details` ; `currency` en est exclue (changée par e04122dc). L'index garde toutes les lignes d'une clé.
- **Écartées** : (a) ne comparer que les identifiants stables : dans l'exemple de #915, ligne à seule `NtryRef`, rien ne les distingue ; (b)
  garder la clé seule : c'est #915.
- **Réversible** : oui, aucune donnée touchée.

### C-e332-4 — Clé d'alias égale, textes différents : importée et signalée

- **Contexte** : le cas qui reste. Soit une ligne neuve (#915), soit la même ligne, dont la banque aurait changé le libellé dans un autre export
  (#922, sous une autre forme). Les deux coûts ne sont pas symétriques : `CLAUDE.md`, *Un appariement automatique propose* : un doublon est
  bruyant, un écart muet ne se voit pas ; et une ligne écartée par erreur manque au relevé sans que rien ne le dise, alors qu'une ligne en double
  se voit à la liste et au solde.
- **Retenu** : la ligne s'importe, l'aperçu la signale (`uncertainLegacyLines`), la trace le compte. Panneau informatif, sans case.
- **Écartées** : (a) écartée par défaut et signalée : le choix `Importer quand même` porte sur **toutes** les lignes, si bien que récupérer la ligne
  neuve réimporterait les vrais doublons ; (b) un choix par ligne : une UI de plus pour un cas très rare ; (c) refuser le relevé : Guy a réservé le
  refus à l'échec de la relecture.
- **Réversible** : oui. **Porté à la connaissance de Guy** au rapport.

### C-e332-5 — Un relevé sans ligne n'est pas refusé

- **Contexte** : l'aperçu ne relit pas un relevé vide (`:1096`), la création si (`:1348`) : sans garde, l'aperçu accepterait ce que la création
  refuse.
- **Retenu** : la fonction rend `Ok(vec![])` si aucune ligne : rien à dédoubler, et les deux sites se comportent pareil.
- **Écartée** : relire à l'aperçu même vide, et refuser : un refus sans objet.
- **Réversible** : oui.

### C-e332-6 — `422 BANK_IMPORT_LEGACY_REREAD_FAILED`

- **Contexte** : les échecs d'analyse rendent `400`, les refus sur un fichier lisible (`NO_MATCHING_STATEMENT`, `DUPLICATE_FILE`) `422`.
- **Retenu** : `422`, le fichier est lisible ; `details.cause` : `reading`, `selection`, `line_count`, `misaligned`.
- **Écartée** : réemployer `BANK_IMPORT_PARSE_FAILED` : le fichier n'est pas en cause pour l'utilisateur, et le message doit dire autre chose.
- **Réversible** : oui avant publication.

### C-e332-7 — Pas de borne dans le temps pour l'alias

- **Contexte** : #915 évoque cette voie. `imported_at` ne dit pas quelle version a lu une ligne, et rien n'enregistre la date de montée.
- **Retenu** : l'alias reste ; C-e332-3 suffit à le rendre sûr.
- **Écartée** : une date en dur : fausse pour toute installation montée plus tard, et chaque erreur serait une ligne en double.
- **Réversible** : oui, le jour où plus aucune base ne porte de ligne d'avant la 0.17.4.

---

## Story 33-3a — les agrégats

### C-e332-8 — Aucune exclusion au titre de la clôture

- **Contexte** : Guy a validé d'écarter les écritures de clôture, et #918 demande de les traiter. Or Kesh n'en passe aucune (manuel `:434`, `:854`,
  `fiscal_years::close`), et rien ne les marquerait. Le résidu : des lignes sur 9000/9100/9200, typés `Expense`, seulement si l'utilisateur a rouvert
  ces comptes, ou écrites avant la Story 24-5.
- **Retenu** : pas de filtre ; un test de caractérisation qui tombera si une clôture écrit un jour.
- **Écartées** : (a) `postable = FALSE` : une vraie charge sur un compte devenu non imputable sortirait en silence du graphique, qui ne concorderait
  plus avec le compte de résultat ; (b) `number LIKE '9%'` : un numéro en dur que la règle de la Story 14-3a interdit au code applicatif, faux sur un plan
  réaffecté.
- **Réversible** : oui. **Prémisse de l'epic corrigée**, à dire à Guy.

### C-e332-9 — Une paire s'écarte des deux côtés, seulement dans le même exercice

- **Contexte** : écarter un seul côté fausse un mois. Une paire vaut zéro par compte (miroir figé, pas de chaîne). Mais `NextFiscalYearStart` existe pour
  les **transitoires**, qui ne sont pas des corrections, et Guy a précisé « contre-passations de corrections ». Rien d'autre que l'exercice ne les distingue.
- **Retenu** : les deux écritures s'écartent si elles sont dans le même exercice ; à cheval, elles restent comptées chacune dans le sien.
- **Écartées** : (a) écarter toute paire où qu'elle soit : le 1er janvier d'une transitoire disparaîtrait, la charge réelle de février compterait, et le
  total de N+1 s'écarterait du compte de résultat, sans que l'utilisateur puisse le recouper ; (b) distinguer par l'audit (`dating`) : lire le journal
  d'audit pour calculer un chiffre comptable, et le cas `Today` à cheval resterait indistinct.
- **Conséquence assumée** : la correction, en janvier, d'une écriture de décembre apparaît en négatif en janvier, comme au compte de résultat.
- **Réversible** : oui, deux conditions SQL. **Porté à la connaissance de Guy** au rapport.

### C-e332-10 — L'invariant

- **Retenu** : le total des barres égale le compte de résultat de l'exercice, à l'unité décimale près, testé.
- **Motif** : un chiffre d'accueil qu'on ne peut pas recouper avec un rapport ne se défend pas. C'est ce que C-e332-9 préserve.
- **Réversible** : il tomberait avec C-e332-9.

### C-e332-11 — Sans trace, sans MCP

- **Contexte** : les rapports tracent `report.generated`. L'accueil se charge à chaque connexion : une trace par visite noierait le journal, et les trois
  tuiles n'en écrivent aucune. R5 prévoit la lecture sans trace.
- **Retenu** : `Role::Consultation` (les droits des rapports), `portee_mcp: None`, aucune écriture.
- **Écartées** : (a) passer par `Rapport<_, Affichage>` : la trace ; (b) `BooksRead` : pas de besoin, et ouvrir un outil MCP est une décision à part.
- **Réversible** : oui.

### C-e332-12 — L'avoir reste compté, les montants sont signés

- **Contexte** : l'avoir a sa propre écriture, peut être partiel, et n'est pas lié par `reverses_entry_id`. C'est un fait commercial, non une correction.
- **Retenu** : il compte à son mois, éventuellement en revenus négatifs.
- **Écartée** : l'apparier à sa facture : un avoir partiel n'a pas de miroir, et le chiffre s'écarterait du compte de résultat.
- **Réversible** : oui.

### C-e332-13 — Le camembert ne porte que les soldes strictement positifs

- **Contexte** : une part négative n'a pas de sens géométrique.
- **Retenu** : écartés du classement, de « Autres » et de la base, comptés dans `nonPositiveExpenseAccounts`.
- **Écartée** : valeur absolue : un compte créditeur ferait paraître une dépense.
- **Réversible** : oui.

### C-e332-14 — Autant de mois que l'exercice

- **Contexte** : rien n'interdit un exercice de 18 mois (`chk_fiscal_years_dates`). « Douze mois » est le cas ordinaire.
- **Retenu** : un élément par mois civil touché, bornes rendues.
- **Écartée** : tronquer à douze : un total faux.
- **Réversible** : oui.

### C-e332-15 — « Aucun exercice ouvert » est un 200

- **Contexte** : un 404 sur un accueil se lit comme une panne. Le jour peut être couvert par un exercice clos.
- **Retenu** : `200`, `state: "noOpenFiscalYear"`, trois cas. Pas de bascule.
- **Écartée** : prendre l'exercice ouvert le plus récent : Guy a dit l'exercice ouvert à la date du jour.
- **Réversible** : oui.

---

## Story 33-3b — l'écran

### C-e332-16 — Le SVG est une image, le tableau porte les données et les liens

- **Contexte** : un `<rect>` cliquable n'est ni annoncé ni atteignable au clavier ; des `<a>` dans un `role="img"` sont présentés puis cachés selon le
  lecteur d'écran.
- **Retenu** : `role="img"`, titre et description chiffrés ; vrais liens au tableau ; `<a tabindex="-1">` pour la souris seulement.
- **Écartée** : des `<a>` focalisables dans le SVG : 12 + 12 arrêts de tabulation sans étiquette utile, et une image qui n'en est plus une.
- **Réversible** : oui.

### C-e332-17 — Jamais la couleur seule

- **Retenu** : trames, position, numéro de rang ; contraste ≥ 3:1 testé par le calcul.
- **Écartée** : étiquettes sur les parts : illisibles sur une petite part, et le tableau les porte.
- **Réversible** : oui.

### C-e332-18 — « Autres » n'est pas cliquable

- **Contexte** : plusieurs comptes, et aucun lien profond existant hors du grand livre (`reports/+page.svelte:569`).
- **Retenu** : pas de lien.
- **Écartée** : le grand livre avec une liste de comptes : la 33-3a devrait rendre leurs identifiants, et un lien de plus à tester, pour une demande que Guy
  n'a pas faite (« un clic sur une part ouvre le grand livre du compte »).
- **Réversible** : oui.

### C-e332-19 — Les valeurs négatives sous l'axe

- **Contexte** : C-e332-12 rend des mois négatifs.
- **Retenu** : axe du zéro dessiné, barres dessous, échelle sur la plus grande valeur absolue.
- **Écartée** : plancher à zéro : un mois négatif paraîtrait nul, et les barres ne recouperaient plus le tableau.
- **Réversible** : oui.

### C-e332-20 — Les bornes des liens

- **Retenu** : mois : `from`/`to` de l'API ; poste : l'exercice entier.
- **Écartée** : poste jusqu'à la date du jour : le camembert porte tout l'exercice, une écriture future manquerait.
- **Réversible** : oui.
