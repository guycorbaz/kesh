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
  base **les textes** de cette lecture, jamais modifiés ensuite (11 requêtes `UPDATE`, aucune sur ces colonnes ; « 17 sites », écrit ici jusqu'à la validation P3, comptait 6 commentaires), et leur calcul n'a pas changé
  depuis l'Epic 8 hors #819 et la devise.
- **Retenu** : égalité stricte sur `value_date`, `reference`, `end_to_end_id`, `transaction_id`, `counterparty_iban`, `counterparty_name`,
  `details` ; `currency` en est exclue (changée par e04122dc). L'index garde toutes les lignes d'une clé.
- **Écartées** : (a) ne comparer que les identifiants stables : dans l'exemple de #915, ligne à seule `NtryRef`, rien ne les distingue ; (b)
  garder la clé seule : c'est #915.
- **Limite écrite** (annoté le 2026-11-06, validation P1, R-4) : #915 est **réduit**, non supprimé. Une ligne neuve dont les sept textes, lus à l'ancienne, égalent ceux d'une ligne existante reste signalée comme doublon : rien ne la distingue
  d'une ligne déjà importée. AC-F5 la fixe, manuel et CHANGELOG la disent, une issue de reliquat est demandée.
- **Réserve écrite** (annoté le 2026-11-06, validation P2) : le choix tient quel que soit le `sql_mode` du serveur. Le pool de production n'en pose aucun (`main.rs:91-94` ; `create_pool` n'a pas d'appelant), `docker-compose.yml` garde le défaut strict de l'image, et `docker-compose.prod.yml` laisse la base à l'utilisateur : le dépôt ne l'établit pas pour le NAS. Sur un serveur sans mode strict, un texte de plus de 255 caractères aurait été tronqué ; la ligne, si elle passe par l'alias, devient incertaine, importée et signalée (C-e332-4), jamais écartée en silence (AC-F4) ; une `reference` elle-même tronquée change la clé et s'importe sans signal, comme en 0.17.4. **Révisé le 2026-11-06, validation P3 (R-6, arbitrage de l'orchestrateur)** : le texte relu se coupe à la largeur de sa colonne avant l'égalité, une seule fois, si bien qu'un texte tronqué à l'écriture reste reconnu au lieu de devenir incertain ; l'exception de la `reference` demeure.
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

### C-e332-21 — À l'aperçu, « déjà importé » passe avant le refus de relecture

*(Ajouté le 2026-11-06, validation P1, R-5.)*

- **Contexte** : un fichier que la 0.17.4 a importé et dont la relecture échoue, redéposé pour vérifier : le refus de relecture lui conseillerait un nouvel export, alors qu'il est déjà dans les livres. La création rendait déjà « déjà importé » d'abord (`:1276`).
- **Retenu** : si la relecture échoue et qu'un import de même empreinte existe, l'aperçu rend `422 BANK_IMPORT_DUPLICATE_FILE`, comme la création sans confirmation. Relecture réussie : inchangé, `200` et avertissement. La confirmation à la création ne lève toujours pas le refus.
- **Écartées** : (a) un `200` avec l'avertissement seul : l'écran offrirait une confirmation vouée au refus ; (b) un message de refus complété : un second texte à traduire pour dire ce que le code voisin dit déjà.
- **Réversible** : oui.

### C-e332-22 — La relecture héritée ne porte pas la borne d'années

*(Ajouté le 2026-11-06, validation P1, M-R4-2, sur arbitrage de l'orchestrateur.)*

- **Contexte** : la 33-4 borne les années des dates reçues, et `parse_date` sert aux deux lectures. Une borne de période coupée (`&#x32;026-10-01…`) se lit en l'an 26, sans erreur, dans la lecture héritée. Avec la borne, ce relevé serait refusé sans raison, et l'inventaire des causes de la 33-2 ne serait plus clos.
- **Retenu** : `parse_legacy_0173` lit comme la 0.17.3, sans borne ; AC-S5 le fixe. La borne se pose à la lecture corrigée, ce que la 33-4 doit dire.
- **Écartée** : laisser la borne jouer et admettre une cause de plus : la lecture ne serait plus celle dont dépend l'égalité des textes (C-e332-3).
- **Réversible** : oui.

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
- **Retenu** : soldes négatifs écartés du classement, de « Autres » et de la base, comptés dans `creditBalanceExpenseAccounts`. **Solde nul** (tranché le 2026-11-06, validation P1, R-7) : absent de tout champ, la requête gardant `HAVING balance != 0`, comme le compte de résultat (`income_statement.rs:83`) ; le champ s'appelait `nonPositiveExpenseAccounts` jusque-là.
- **Écartée** : valeur absolue : un compte créditeur ferait paraître une dépense. Écartée aussi : compter les soldes nuls : la note parlerait de comptes que le compte de résultat ne montre pas.
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
  lecteur d'écran, et le contrôle `no-focusable-content` d'axe refuse tout descendant focalisable de rôle widget, donc un `<a href>`, `tabindex="-1"` compris (`axe.js:25720-25757`, `:16037-16039`, lu, non joué).
- **Retenu** (récrit le 2026-11-06, validation P1, R-1) : `role="img"`, titre et description chiffrés, **aucun élément focalisable dans le SVG** ; vrais liens au tableau voisin ; pour la souris, un écouteur unique sur le `<svg>` qui lit le `data-href` de la marque. Un squelette passe sous axe à l'étape 0.
- **Écartées** : (a) des `<a>` focalisables dans le SVG : 12 + 12 arrêts de tabulation sans étiquette utile, et une image qui n'en est plus une ; (b) `<a tabindex="-1">` dans un `role="img"`, version d'origine de ce choix : voir le contexte ; (c) deux calques, l'un sous `aria-hidden` avec les `<a tabindex="-1">` : deux dessins à tenir alignés, pour le seul `Ctrl`+clic, que le tableau offre ; (d) des liens HTML superposés : un rectangle ne suit pas une part de camembert.
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

### C-e332-23 — Un exercice long : tous ses mois, moins d'étiquettes, aucun défilement

*(Ajouté le 2026-11-06, validation P1, R-2. C-e332-14 n'est pas rouvert. Simplifié le même jour, validation P2, R-7 et R-8 : quatre tranches et une distance minimale testée, qui ne prouvait pas la lisibilité.)*

- **Contexte** : rien ne borne la durée d'un exercice, et la base E2E en a un de 132 mois (`test_fixtures.rs:116-121`). Ces exercices n'existent presque pas en production : la règle reste simple.
- **Retenu** : un groupe par mois, sans `overflow-x` ; au-delà de 12 mois, carte pleine largeur ; étiquettes tirées du mois rendu : jusqu'à 12 mois, chacun ; de 13 à 36, chaque début de trimestre civil ; au-delà, chaque janvier. La lisibilité se regarde aux captures. La spec E2E des graphiques crée son exercice de douze mois.
- **Limite écrite** : à partir d'une soixantaine de mois (estimation, non jouée), la trame d'une barre ne se voit plus ; seule la position sépare revenus et dépenses, et le tableau porte les chiffres.
- **Écartées** : (a) défilement horizontal : exige un élément focalisable (`scrollable-region-focusable`) et cache des mois ; (b) agrégation par trimestre ou par année au-delà d'un seuil : détache les barres des liens de mois, et le contrat rend des mois ; (c) changer le preset `with-company` : d'autres specs comptent sur sa largeur (« dérive d'horloge CI ») ; (d) mesurer la largeur du texte en test : une exigence de plus pour des exercices rares.
- **Réversible** : oui, frontend seul.

### C-e332-24 — Barres et camembert décident chacun sur sa seule donnée

*(Ajouté le 2026-11-06, validation P2, R-2.)*

- **Contexte** : des montants mensuels nuls n'impliquent pas l'absence de poste. Un reclassement entre deux comptes de charges (débit 6000, crédit 6100) laisse `months` à zéro et un poste à 100 %, et un état « exercice vide » commun aux deux graphiques masquait cette part en affirmant une phrase fausse.
- **Retenu** : les barres décident sur `months`, avec une phrase juste dans tous ces cas (« Aucun mouvement net de produits ou de charges dans cet exercice ») ; le camembert décide sur `expensePosts`, avec une seule phrase pour le vide (« Aucun poste de charges à répartir dans cet exercice »), elle aussi juste dans tous les cas.
- **Écartées** : (a) distinguer au camembert « exercice vide » et « sans poste » : une condition de plus pour deux phrases vraies toutes les deux ; (b) rendre un indicateur de lignes par l'API, pour séparer un exercice réellement vide d'un reclassement de produits : un champ de contrat pour une nuance de libellé.
- **Réversible** : oui, frontend seul.
