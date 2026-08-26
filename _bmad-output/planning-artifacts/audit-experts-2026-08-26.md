# Audit professionnel de Kesh — 26 août 2026

## Ce que ce document est, et ce qu'il n'est pas

Trois avis professionnels ont été demandés sur **l'ensemble** de Kesh (v0.11.1, 23 epics
clos, 156 stories), après qu'un premier examen eut établi un défaut de fond dans le cycle
d'encaissement client. Les trois ont travaillé **en parallèle, avec des mandats disjoints**,
et chacun a vérifié ses affirmations dans le code.

| profil | mandat |
|---|---|
| **Expert-comptable** suisse | les livres sont-ils tenables et bouclables ? |
| **Expert fiscal** suisse | que se passerait-il en cas de contrôle ? |
| **Directeur financier** | peut-on **piloter** une activité avec cet outil ? |

⚠️ **Ce document consigne l'analyse. Il ne tient AUCUNE liste de défauts** — la § *Issue
Tracking Rule* du `CLAUDE.md` pose que GitHub Issues est l'unique source de vérité et
interdit tout suivi parallèle dans le dépôt. Chaque défaut nommé ici a, ou aura, son issue ;
ce texte porte le **raisonnement**, pas le suivi.

## Les quatre profils d'usage jugés

Le PRD annonce « personal and small business accounting ». Le Project Lead a précisé quatre
usages, et les trois experts les ont jugés séparément :

1. **Comptabilité privée d'un particulier** — cible : une personne **propriétaire de
   bâtiments**, avec revenus annexes et investissements. Deux besoins déclarés : **budgets
   pluriannuels** et **collecte des déductions** pour la déclaration.
2. **Raison individuelle** (Einzelfirma).
3. **Indépendant**.
4. **PME** (Sàrl / SA) avec salariés.

---

## I. Ce que les trois experts jugent SOLIDE

À consigner en premier, parce que l'audit a été demandé dans un moment de doute sur
l'ensemble du logiciel, et que ce doute n'est pas justifié partout.

**Le moteur de partie double est meilleur que celui de beaucoup de produits commerciaux**
(expert-comptable). Triple défense sur l'équilibre : validation métier
(`kesh-core/src/accounting/balance.rs`), contrainte SGBD `chk_jel_debit_credit_exclusive`,
et recalcul `SUM(debit) = SUM(credit)` après insertion avec rollback.

**Le verrou d'exercice est appliqué aux onze chemins d'écriture, sans exception** —
*« Je n'ai trouvé aucune porte dérobée. »* La réouverture est réservée à l'Admin, interdite
aux clés API, exige un motif, est auditée, et applique une garde LIFO.

**Les rôles de comptes ne se déduisent jamais du numéro** (`20260722000001_accounts_role_postable.sql`) :
le plan reste librement renumérotable — *« une décision d'architecture juste, et rare »*.

**L'arithmétique est irréprochable** : `rust_decimal` partout, `DECIMAL(19,4)`, arrondi TVA
**par ligne** conforme à la pratique AFC, avec un test explicite interdisant d'arrondir une
base agrégée. Le fiscaliste : *« c'est irréprochable, et c'est rare »*.

**Le mono-CHF est assumé et gardé, pas subi** : une transaction non-CHF est **refusée**,
jamais convertie au hasard.

**La facturation est de bon niveau.** QR-facture SIX 2.2 avec adresses structurées, avoirs,
envoi multilingue, relances multi-niveaux avec frais, import CAMT.053, rapprochement avec
règles, pain.001. Le directeur financier : *« sur ce périmètre, Kesh soutient la comparaison
avec des produits payants »*.

**La balance âgée inclut délibérément les factures dont la relance est suspendue** —
*« exactement la garde qu'un directeur financier veut : on ne masque pas une créance en
suspendant sa relance »*.

**L'analytique par projet est un vrai instrument**, et la seule prise réelle du profil 1 :
un immeuble = un projet, avec un rendement cumulé qui, **seul dans toute l'application**,
franchit la borne d'exercice.

**Le plan `independant.json` est écrit par quelqu'un qui connaît le métier** : prélèvements
et apports privés, AVS de l'exploitant, part professionnelle du loyer, véhicule et son
amortissement cumulé.

**La conservation des justificatifs entrants est solide** : fichiers nommés par SHA-256,
déduplication par `UNIQUE(company_id, file_hash)`, chaîne document → écriture protégée par
`ON DELETE RESTRICT`.

---

## II. Le défaut fondateur — le cycle d'encaissement client

Établi avant l'audit, confirmé par les trois.

**Côté fournisseur, le cycle est complet** : `pay_in_tx`
(`kesh-db/src/repositories/supplier_invoices.rs`) crée une seconde écriture
`D 2000 / C contrepartie` — banque ou compte interne — et pose `paid_at`.

**Côté client, rien.** Ni le marquage manuel (`mark_as_paid`,
`kesh-db/src/repositories/invoices.rs:1922`, dont le commentaire dit *« Ne crée AUCUNE
écriture comptable »*), ni la réconciliation bancaire (`accept_one_invoice`,
`kesh-api/src/routes/reconciliation.rs:1003-1313`, qui rapproche la transaction de
**l'écriture de vente**).

⚠️ **Le mode d'échec est silencieux, et c'est ce qui le rend grave.** Le bilan reste
**équilibré** — la partie double est respectée — mais deux postes sont faux du même montant
en sens inverse : **débiteurs surévalués**, **banque sous-évaluée**. Aucun contrôle interne
ne rougit.

**Conséquences établies :**

- **le rapprochement bancaire devient impossible** — l'écart au 31.12 vaut le total des
  encaissements clients depuis l'origine. C'est le premier contrôle d'un réviseur ;
- **le lettrage client n'a rien à lettrer** : une facture payée n'a qu'une ligne au compte
  débiteurs ;
- **le seul chiffre du tableau de bord est faux** : le « solde bancaire » de la page
  d'accueil est le solde **comptable** (`bank_accounts.rs:649`), donc entaché du même
  défaut, sans que rien ne l'indique.

⚠️ **Et les deux chemins existants s'excluent.** `post_manual` permet d'obtenir la bonne
écriture en désignant le compte débiteurs comme contrepartie — mais `paid_at` n'est alors
pas posé, donc la facture **reste relançable**. Ou bien la comptabilité est juste et le
client reçoit un rappel pour une facture payée, ou bien le suivi est juste et les livres sont
faux. **On ne peut pas avoir les deux aujourd'hui.**

**Arbitrage du Project Lead (2026-08-26)** : *« Une facture est payée parce que la
comptabilité le montre. »* `paid_at` devient donc la **projection** de l'existence d'une
écriture de règlement, et non une donnée qu'on écrit.

---

## III. Ce qui rend les livres non opposables

### III.1 Il n'existe aucun grand livre

⚠️ **Les trois experts le citent ; le comptable et le directeur financier le placent en tête
de leurs priorités, indépendamment l'un de l'autre.**

Aucun extrait de compte parmi les huit rapports livrés, **et aucun contournement** : la liste
des écritures se filtre par libellé, date, journal et fourchette de montant — **pas par
compte** (`ListJournalEntriesQuery`, `kesh-api/src/routes/journal_entries.rs`). Aucun
drill-down depuis le bilan, le compte de résultat ou la balance.

La balance dit que le compte débiteurs vaut 84 320.15 : **rien dans Kesh ne permet de savoir
de quoi c'est fait**. C'est la première pièce que demande un réviseur, et c'est aussi ce qui
rendrait visibles les autres défauts — d'où sa place en tête.

Le manuel administrateur affirme pourtant *« grand livre + journal → implémentés »*.

### III.2 Une écriture est réécrivable et destructible sans trace opposable

- le `PUT` réécrit date, journal, libellé **et toutes les lignes** — comptes et montants —
  par `DELETE` puis réinsertion (`journal_entries.rs:981-1023`) ;
- le `DELETE` est **physique** ; aucun `deleted_at` n'existe dans le schéma ;
- **aucun statut brouillon / comptabilisé** : la table n'a pas de colonne `status` ;
- **aucune contre-passation d'écriture manuelle** — l'extourne n'existe qu'à l'intérieur des
  flux documentaires (avoir, annulation fournisseur) ;
- **le seul verrou est la clôture annuelle** : rien ne fige un trimestre TVA déclaré ;
- ⚠️ **la numérotation admet des trous ET la réutilisation** : supprimer la dernière écriture
  fait **réattribuer son numéro** à une écriture au contenu différent. Le commentaire de la
  migration qui promet *« jamais de trou »* est **faux**.

Au regard de l'art. 958f CO et de l'Olico art. 3, l'exigence n'est pas qu'on ne se trompe
jamais : c'est que **la correction soit apparente**.

### III.3 La seule trace de ces corrections est effaçable — et illisible

C'est ce qui fait passer III.2 d'un choix d'ergonomie discutable à un défaut de conformité.

Le module d'audit déclare : *« Pas de méthode delete : CO art. 957-964 impose la conservation
10 ans. Les entrées sont inamovibles. »* (`kesh-db/src/repositories/audit_log.rs:3-4`).
**Cette phrase n'est pas tenue.**

- `audit_log` figure dans `TABLES_TO_TRUNCATE` (`backup.rs`) : l'import d'un `.keshbackup`
  **remplace intégralement** la piste de contrôle ;
- ⚠️ **`reset_demo` exécute un `DELETE FROM audit_log` non scopé** (`kesh-seed/src/lib.rs:250`),
  et sa route est montée dans le bloc **« tout rôle authentifié »** (`kesh-api/src/lib.rs:749`),
  sans `require_admin_role`. Sa seule protection est **un état, pas un droit** ;
- aucun déclencheur ni `REVOKE` au niveau SGBD — zéro résultat sur les 61 migrations ;
- ⚠️ **aucune route, aucun écran ne permet de le consulter.**

*« Apparent » et « archivé dans une table que personne ne peut lire » ne sont pas la même
chose.* S'y ajoutent des trous d'alimentation : ni la gestion des utilisateurs ni la
modification de la société ne sont tracées — un changement de rôle vers Comptable ne laisse
aucune trace.

### III.4 Les états financiers ne satisfont pas le minimum légal

| exigence | état |
|---|---|
| **art. 958d al. 2 CO** — chiffres de l'exercice précédent | **absents** — aucune structure ne porte de comparaison |
| **art. 959c CO** — annexe aux comptes annuels | **inexistante** — aucune route, aucun écran |
| **art. 959a CO** — structure minimale du bilan | liste plate de comptes triée par numéro ; aucune distinction circulant / immobilisé, court / long terme. La hiérarchie `parent_id` existe et **n'est utilisée par aucun rapport** |
| **art. 959b CO** — structure du compte de résultat | deux listes plates et un résultat net ; **aucune** des onze rubriques légales |

⚠️ **Conclusion de l'expert-comptable** : les comptes annuels ne peuvent être remis tels
quels ni à un réviseur, ni à une banque, ni au fisc — **donc le bouclement ne se fait pas
dans Kesh**.

Le manuel promet pourtant les présentations « par nature » et « par fonction » avec un choix
par société. Elles n'existent pas.

### III.5 Les comptes de clôture de classe 9 sont typés `Expense` et postables

Les trois plans livrés portent `9000 Bilan d'ouverture`, `9100 Compte de résultat`,
`9200 Bilan de clôture`, **typés `Expense`**. Le compte de résultat sélectionne **uniquement
sur `account_type`**, et ces comptes étant des feuilles, ils restent **postables**.

⚠️ Un utilisateur venant d'un autre logiciel — précisément la persona du bilan d'ouverture —
qui passe son à-nouveau par le 9000 verra **le montant entier de son bilan atterrir en
charges**. L'équation du bilan continue de tenir, rien ne l'avertit, et le résultat est faux
du montant du bilan.

**Correctif : une ligne dans trois fichiers JSON, plus un backfill.** L'expert-comptable ne
le classe hors de ses trois priorités que parce que le risque est conditionnel — c'est le
meilleur rapport coût/effet du dossier.

### III.6 Le type d'un compte mouvementé est modifiable, sans garde

`UPDATE accounts SET … account_type = ? …` (`accounts.rs:415`). Le numéro est immuable —
bonne décision — mais rien ne vérifie que le compte est vierge de mouvements avant d'en
changer le type. Passer un compte de charge en actif **reclasse rétroactivement tout son
historique, exercices clos compris**.

⚠️ **C'est le seul endroit du dépôt où la clôture est contournable sans réouverture.**

---

## IV. La fiscalité — le risque le plus grave du dossier

### IV.1 Le décompte TVA n'est rattaché à rien, et son contrôle ne peut pas échouer

**C'est le risque n°1 selon le fiscaliste.** Trois défauts se composent :

1. **la TVA due est dérivée du statut des documents** (`i.status = 'validated'`,
   `kesh-report/src/vat_report.rs:90-98`) — donnée **mutable** — et non du grand livre ;
2. **un avoir mute ce statut rétroactivement** : l'émission bascule la facture d'origine en
   `cancelled` (`credit_notes.rs:563`) et date la contre-passation du jour de l'avoir. Une
   facture de mars créditée en août **disparaît du rapport de Q1** — qui a déjà été déclaré —
   **sans réapparaître en Q3**, les avoirs vivant dans une table que le rapport ne lit pas.
   L'art. 41 al. 1 LTVA impose au contraire de corriger **dans la période où la réduction est
   constatée** ;
3. ⚠️ **le contrôle de cohérence applique le MÊME filtre aux deux membres** de la comparaison
   (`vat_report.rs:218-227`) : l'écriture de l'avoir est invisible des deux côtés, les deux
   membres tombent à zéro ensemble, et le rapport affiche **`reconciliation_status: "ok"`**.

⚠️ **Un test du dépôt valide ce comportement** et affirme que « la réconciliation reste
cohérente après avoir ». Le fiscaliste : *« ce n'est pas une réconciliation, c'est une
tautologie »*.

**Le résultat n'est pas un chiffre faux : c'est un chiffre faux accompagné d'une attestation
de cohérence.** Et comme aucune période n'est verrouillée et qu'aucun décompte déposé n'est
conservé, nul ne peut établir six mois plus tard ce qui avait été déclaré. En contrôle, cela
ne se solde pas par une reprise ponctuelle mais par une **taxation par estimation**
(art. 79 LTVA), la comptabilité cessant d'être probante au sens de l'art. 70 al. 1.

**Corollaires** : la contrainte `UNIQUE (invoice_id)` sur les avoirs n'autorise **qu'un seul
avoir total par facture** — rabais, escompte et note de crédit partielle sont hors
d'atteinte ; et **un avoir peut annuler une facture d'un exercice clos**, alors que sa
suppression est correctement refusée. L'asymétrie est nette, et le manuel affirme le
contraire.

### IV.2 L'impôt préalable est déduit à 100 %, sans aucune notion de déductibilité

Kesh débite l'impôt préalable de la **totalité** de la TVA de toute ligne d'achat. Sont
absents **en totalité** : art. 29 (prestations exclues), art. 30 (double affectation),
art. 31 (prestation à soi-même), art. 32 (dégrèvement ultérieur), art. 33 al. 2 (subventions).

⚠️ **Le fiscaliste appelle cela « le redressement de manuel »** : tout assujetti ayant du
chiffre d'affaires exclu — location, formation, santé, finance — ou un usage privé
sur-déduit **mécaniquement**, sans alerte ni champ où consigner la correction. Reprise sur
cinq ans, avec intérêt moratoire.

### IV.3 Un seul « 0 % » pour trois régimes juridiquement incompatibles

`exempt 0.00` recouvre : **exonéré** (art. 23 — taux zéro **avec** droit à déduction),
**exclu** (art. 21 — **sans** droit, et déclenchant la correction de l'art. 30), et **hors
champ** (subventions, dommages-intérêts). Effets **diamétralement opposés** sur le décompte
et sur le droit à déduction. Deux commentaires du code confondent d'ailleurs les notions
(*« 0 % / hors champ »*).

⚠️ Un bailleur assujetti pour une partie de son activité — cas courant, **et directement
pertinent pour le profil 1** — n'a aucun moyen de produire un décompte correct.

*Point positif : la colonne `category` est volontairement ouverte, sans `CHECK IN (…)`. Le
socle pour corriger existe.*

### IV.4 Les taux ne sont pas contrôlés à la date de la prestation

`vat_rates` porte bien `valid_from` / `valid_to`, et un accesseur temporel existe —
`find_for_category_at_date`. ⚠️ **Il n'est appelé nulle part en production** : ses huit
occurrences sont toutes dans les tests. La validation branchée sur la facturation ignore la
date.

*Le mécanisme de datation existe, le contrôle qui l'utiliserait n'existe pas.* Le régime
suisse a changé de taux en 2018 et en 2024 ; au prochain, le défaut deviendra actif.

### IV.5 Le côté achats est plus faible que le côté ventes

- **aucun contrôle du taux** : borné au seul bon sens arithmétique (0 à 100 %) — on peut
  déduire un impôt préalable à 13,7 % ;
- **l'assistant d'achat est purement frontal** : la route backend ne connaît aucune logique
  TVA. Le solde du compte d'impôt préalable — **source unique de la déduction dans le
  rapport** — est exactement ce qu'un utilisateur y a porté, par n'importe quel chemin ;
- **la saisie se fait en HT, jamais en TVA facturée**, alors que l'art. 28 al. 1 LTVA ouvre
  le droit à déduction sur **l'impôt qui a été facturé** ;
- **aucun mode de prix TTC**, alors que l'art. 26 al. 3 l'autorise — un commerce de détail ne
  peut pas facturer comme il affiche.

### IV.6 Le décompte n'est jamais comptabilisé

Le compte de décompte TVA (rôle `VatSettlement`) est **déclaré partout et jamais mouvementé**.
Il n'existe aucun code qui vire le solde de la TVA due et de l'impôt préalable vers lui en fin
de période. ⚠️ **Les deux comptes s'accumulent d'exercice en exercice** : la TVA due affichée
au bilan est le **cumul depuis l'origine**, pas la dette réelle envers l'AFC.

### IV.7 Régimes absents

| régime | base légale | état | le manuel le dit ? |
|---|---|---|---|
| contre-prestations convenues | art. 39 al. 1 LTVA | **seul régime servi** | — |
| contre-prestations reçues | art. 39 al. 2 | hors d'atteinte | non |
| **taux de la dette fiscale nette (TDFN)** | art. 37 | **absent** | oui |
| taux forfaitaires | art. 37 al. 5 | absent | non |
| **impôt sur les acquisitions** | art. 45 | **absent** | non |
| e-décompte AFC / format AFC | art. 65a | absent | oui |

⚠️ **Le PRD promet la méthode forfaitaire. Elle n'existe pas.** Or la TDFN est **le** régime
de la petite entreprise suisse — jusqu'à 5 024 000 CHF de chiffre d'affaires —, c'est-à-dire
de la cible déclarée.

⚠️ **L'impôt sur les acquisitions est l'omission la plus inquiétante** : toute PME qui achète
un service à un prestataire étranger — un abonnement logiciel suffit — en est redevable dès
10 000 CHF par an, **même non assujettie par ailleurs**. C'est ce sur quoi l'AFC redresse le
plus les petites structures, et **il n'existe pas même un champ pour le consigner**.

### IV.8 Conservation — l'intégrité repose sur une empreinte que le contribuable calcule
lui-même

Aucun chaînage cryptographique ni horodatage qualifié. La procédure du manuel est
entièrement manuelle : générer le ZIP, faire `sha256sum`, stocker, revérifier.

⚠️ L'Olico (art. 9 al. 1 let. b) n'admet la conservation sur support modifiable
qu'accompagnée de procédés garantissant l'intégrité. *Une empreinte calculée et conservée
**par la personne même dont elle est censée contraindre le comportement** ne constitue pas
cette garantie : qui modifie les données recalcule l'empreinte.*

**Ce n'est pas une conformité Olico, c'est une bonne pratique d'archivage** à laquelle il
manque le tiers de confiance.

### IV.9 Le PDF de facture n'est jamais archivé

Il est **régénéré à la volée** (`kesh-api/src/routes/invoice_pdf.rs:23`). Une facture
réimprimée dans cinq ans **ne sera pas celle qui a été remise au client**.

---

## V. Le pilotage — ce qui manque pour diriger

### V.1 Aucune prévision de trésorerie, aucun budget

Ni tableau de flux, ni plan de liquidités, ni projection. Le budget est **spécifié depuis
l'origine** — Epic 12/13, versions par exercice, saisie mensuelle, rapport d'écart — et
**jamais exécuté**, classé en backlog `v0.4-milestone` alors que le projet est en v0.11.
C'est le seul epic d'origine encore en attente.

⚠️ **C'est le besoin n°1 déclaré du profil 1.**

### V.2 Aucun rapport ne franchit la borne d'exercice

`ReportPeriod` porte un `fiscal_year_id` **obligatoire** et **refuse** toute borne qui en
sort. Conséquences : **aucune comparaison N-1** (qui est aussi une exigence légale, cf.
III.4), **aucune vue pluriannuelle** — donc rien de ce que le profil 1 demande —, et aucune
évolution mensuelle sans douze allers-retours manuels.

*Seul le rapport « Rendement par projet » y échappe.*

### V.3 Le tableau de bord ne dit rien, et l'une de ses tuiles est un décor

La tuile « Dernières écritures » **n'appelle rien** : elle affiche inconditionnellement
« Aucune écriture », quelle que soit la base. La tuile « Factures ouvertes » n'affiche jamais
de montant. Et le seul chiffre réel — le solde bancaire — est faux (cf. II).

### V.4 Les dettes fournisseurs n'ont aucun outil de gestion

`ListSupplierInvoicesQuery` **ne contient que `limit` et `offset`** : pas de filtre par
statut ni par échéance, pas de tri, pas de total. Aucune balance âgée fournisseurs. ⚠️ **Et
aucune notion d'escompte** — zéro occurrence dans tout le dépôt : un 2 % à 10 jours se perd
par ignorance. Le lot de paiement porte **une** date d'exécution unique : on regroupe, on ne
planifie pas.

### V.5 Aucun indicateur de gestion

Ni marge, ni seuil de rentabilité, ni besoin en fonds de roulement, ni délai moyen
d'encaissement, ni provision pour créances douteuses.

### V.6 L'export de souveraineté est incomplet, et il promet plus qu'il ne tient

L'écran annonce : *« archiver, **migrer vers un autre logiciel**, conserver vos données
10 ans (art. 958f CO) »*. Il exporte **19 tables sur ~38**.

**Absents** : `supplier_invoices` et leurs lignes — **toutes les factures fournisseurs** —,
`credit_notes` — **tous les avoirs** —, `projects` — si bien que `project_id` sort en
**identifiants orphelins** —, `payment_batches`, `contact_persons`, `audit_log`, et les
**justificatifs** ne sont dans aucune sauvegarde.

⚠️ Trois endroits annoncent trois décomptes différents (18, 19, 16). La bonne valeur est 19.

*« Si je quitte Kesh, j'emporte mes ventes et ma banque, et je laisse mes achats derrière
moi. »*

---

## VI. Ce qui manque, par objet

Aucun des objets suivants n'existe : **amortissements** et registre d'immobilisations,
**provisions**, **actifs et passifs transitoires**, **stocks** (la table `products` n'a
aucune quantité — c'est un catalogue tarifaire), **écritures récurrentes**, **salaires et
charges sociales**, **budgets**, **devises**, **multi-entités et consolidation**.

⚠️ **Les comptes correspondants figurent aux plans** (1200, 1300, 2300, 2500, 1500/1509,
6800…). **La présence du compte n'est pas la présence de la fonction** — et c'est un piège
d'apparence : le plan donne l'impression d'un logiciel complet.

---

## VII. Verdict par profil — les trois experts

### Profil 1 — particulier propriétaire : **non servi, et jamais visé**

**Unanimité des trois.** Le profil n'apparaît dans **aucun** persona du PRD (un graphiste,
une trésorière d'association, un auto-hébergeur, une comptable de fiduciaire), aucun epic,
aucun plan comptable — il n'en existe que trois : PME, association, indépendant.

⚠️ **Un PRD antérieur, abandonné, le contenait** : persona « particulier organisé », plan
comptable « perso », comptabilité personnelle. Les trois ont disparu sans être implémentés.

**Ses deux besoins déclarés sont précisément les deux qui n'existent pas** : le budget
pluriannuel (spécifié, au backlog, zéro ligne de code) et la collecte des déductions.
Manquent également : l'objet **immeuble**, la **valeur locative**, les **intérêts
hypothécaires**, le choix **forfait / effectif** des frais d'entretien, la distinction
**entretien déductible / impense d'amélioration** — celle dont dépend la déduction —, l'**état
des titres** et l'**impôt anticipé**, le **pilier 3a**, les primes, les frais médicaux.

⚠️ **Et Kesh lui impose la partie double**, dont il n'a aucune obligation (l'art. 957 CO ne
le vise pas). Ce qu'il lui faut est un relevé de flux catégorisés et un état de fortune.

**Ce qu'il obtient réellement** : le rapport « Dépenses par projet », né de son besoin — le
document de design cite le propriétaire mot pour mot. Utile, mais il exige de taguer chaque
ligne à la main et **ne distingue pas l'entretien déductible de l'impense d'amélioration**.

> **Ce n'est pas un epic de retard : c'est une autre application, greffée sur le même
> moteur.** (expert-comptable)

### Profils 2 et 3 — raison individuelle et indépendant : **les mieux servis**

C'est la cible réelle du PRD, et Kesh y tient à peu près sa promesse. **Utilisable comme
livre de base**, sous conditions cumulatives (fiscaliste) : assujetti selon la **méthode
effective** — la TDFN étant absente —, **sans chiffre d'affaires exclu**, **sans usage privé**,
**sans devises**, **sans achat de services à l'étranger**.

Restent manuels ou externes : amortissements, provisions, ducroire, part privée,
transitoires, et la restructuration des états financiers.

> **Il tient les livres, il ne tient pas la fiscalité.**

### Profil 4 — PME : **non utilisable seul, et c'est le profil sur lequel les experts mettent
en garde**

Trois obstacles dirimants : **aucune paie** — alors que le profil se définit par la présence
de salariés —, **aucune annexe** (art. 959c CO, imposée à toute personne morale), et un
**compte de résultat sans la structure de l'art. 959b**. S'y ajoutent l'absence de charge et
provision d'impôt, d'impôt sur le capital, de traitement des dividendes et de l'impôt
anticipé, le **compte courant associé absent de `pme.json`** alors qu'il est dans les deux
autres plans, et le monolinguisme CHF.

⚠️ **Pourquoi c'est ce profil-là qu'il faut signaler, et non le profil 1** : le profil 1 est
absent et **se voit** ; personne ne s'y trompera. La PME est **partiellement servie** — la
facturation, la QR-facture, l'import bancaire, l'analytique sont bons. *« C'est ce qui rend
le piège efficace : on tient neuf mois avec le sentiment que tout marche, et on découvre au
bouclement que rien de ce qu'il faut n'existe. »*

⚠️ **Et c'est le seul des quatre profils jugé par un tiers** — fiduciaire, réviseur, banque —
qui demandera exactement les quatre pièces absentes : grand livre, annexe, comparatifs,
justification des soldes.

---

## VIII. Le motif transversal : la documentation promet ce que le code ne fait pas

Relevé par les trois, et il ne porte pas sur des détails.

| ce qui est écrit | où | réalité |
|---|---|---|
| cycle « Brouillon → Validée → Annulée », bouton Valider, suppression impossible après validation, contre-passation automatique | manuel utilisateur | **rien de tout cela n'existe** |
| « les entrées sont inamovibles » | module d'audit | effaçable par **deux** routes |
| « grand livre + journal → implémentés » | manuel admin | pas de grand livre |
| « audit_log immutable insert-only » | manuel admin | faux |
| « intangibilité des périodes clôturées préservée » | manuel utilisateur | un avoir modifie un exercice clos |
| présentations « par nature » et « par fonction » | manuel utilisateur | n'existent pas |
| « comptabilité simplifiée (recettes-dépenses) » | brochure | n'existe pas |
| « gérez plusieurs sociétés sur une seule instance » | brochure | aucune route de bascule |
| « migrer vers un autre logiciel » | écran d'export | 19 tables sur 38 |
| méthode « forfaitaire » | PRD | absente |

⚠️ **Le danger propre, relevé par le fiscaliste** : ce manuel est destiné à être **produit
devant l'AFC** comme description du système de contrôle interne — usage que l'Olico prévoit
et que le manuel lui-même recommande. *Un contribuable qui le remet décrit un dispositif qui
n'est pas le sien.*

⚠️ **Ce motif n'est pas nouveau dans ce projet** : le `CLAUDE.md` porte lui-même la trace
d'un paragraphe qui *« a menti pendant des mois »* en affirmant qu'aucun code applicatif
n'avait été écrit. **Le défaut n'était pas isolé, il est systémique** — et c'est le seul
enseignement de cet audit qui porte sur la méthode plutôt que sur le produit.

---

## IX. Où les trois experts divergent

Uniquement sur **l'ordre**, et chacun a raison depuis son mandat :

| expert | sa priorité n°1 | son argument |
|---|---|---|
| **Expert-comptable** | le **grand livre** | *« c'est l'instrument qui rend les autres défauts visibles »* |
| **Fiscaliste** | la **TVA** | l'échéance revient chaque trimestre, et un contrôle est irréversible |
| **Directeur financier** | **trésorerie et budget** | *« on ne dirige pas sans eux »* |

⚠️ **Ils s'accordent en revanche sur le rang zéro** : l'**écriture d'encaissement**, sans
laquelle rien d'autre ne tient.

---

## X. Ce que chacun ferait en premier

**Expert-comptable** : le grand livre ; fermer la porte à la correction invisible (statut
comptabilisé, contre-passation au lieu de réécriture, verrou de période, audit inaltérable
et consultable) ; comptabiliser le décompte TVA et rendre le rapport reproductible.
*Plus, « pour un coût dérisoire » : dépostabiliser les comptes de classe 9.*

**Fiscaliste** : dériver la TVA due du **grand livre** et non du statut des documents ;
traiter l'avoir comme une correction de **sa** période (art. 41) ; **aligner le manuel sur
le code** — *« la moins coûteuse des trois, et celle qui protège aujourd'hui le moins bien »*.

**Directeur financier** : l'encaissement ; le grand livre et le filtre par compte ; un
échéancier fournisseurs digne de ce nom ; une prévision de trésorerie à 90 jours ; la
comparaison N-1 ; l'export complété — *« ou, à défaut, le retrait de la promesse : une
promesse de réversibilité à moitié tenue est pire qu'une absence de promesse, elle décourage
de prendre ses précautions »*.

---

## X-bis. La priorité de cible arbitrée par le Project Lead — 2026-08-26

> **1. Comptabilité personnelle — 2. Indépendant — 3. PME.**

⚠️ **Cet ordre INVERSE le degré de maturité constaté par les trois experts.** Le profil le
mieux servi (indépendant) passe en deuxième, et **le profil qu'aucun epic n'a jamais visé
passe en premier** — celui dont l'expert-comptable écrit : *« ce n'est pas un epic de retard,
c'est une autre application, greffée sur le même moteur »*.

C'est un arbitrage légitime — Kesh est d'abord l'outil de son auteur, et le profil 1 est le
sien. Mais il a **quatre conséquences** qu'il faut inscrire, faute de quoi le plan d'action
hérite d'un ordre de priorité qui n'est plus le bon.

**(a) La TVA cesse d'être le risque n°1.** Le fiscaliste la classait en tête *pour un
assujetti*. Un particulier ne l'est pas, et la location d'immeuble est **exclue du champ**
(art. 21 al. 2 LTVA). Le risque TVA reste entier — il ne disparaît pas —, mais il devient le
risque du **profil 2**, donc de la deuxième vague de cibles, pas de la première.

**(b) Ce que le profil 1 exige n'est dans AUCUNE vague** : budgets pluriannuels, objet
immeuble, valeur locative, intérêts hypothécaires, choix forfait/effectif, distinction
entretien déductible / impense d'amélioration, état des titres et impôt anticipé. Le plan en
quatre vagues corrige **l'existant** ; le profil 1 demande de **construire ce qui n'existe
pas**. Ce sont deux chantiers de nature différente, et le second ne se déduit pas du premier.

**(c) La partie double imposée devient une question ouverte.** L'expert-comptable relève
qu'un particulier n'a **aucune obligation comptable** — l'art. 957 CO ne le vise pas — et que
lui imposer un bilan pour reconstituer ses déductions est **disproportionné**. Ce qu'il lui
faut est un relevé de flux catégorisés et un état de fortune. ⚠️ **La brochure promet déjà
une « comptabilité simplifiée (recettes-dépenses) »** qui n'existe pas : la vague 0 devra
soit retirer cette promesse, soit la tenir — et si la priorité est le profil 1, la tenir
devient un chantier de premier rang.

**(d) La recommandation des experts sur la PME est confirmée par cet ordre.** Placer la PME
en troisième position revient à admettre qu'elle n'est pas servie aujourd'hui — ce que les
deux experts demandaient de dire publiquement.

## XI. Le plan d'action retenu

Arbitré par le Project Lead le 2026-08-26, en quatre vagues.

| vague | objet | principe |
|---|---|---|
| **0** | **La vérité** | aligner manuels, brochure, site et README sur ce que le code fait. **Zéro développement.** |
| **1** | **Les livres justes** | l'encaissement client et son écriture, le grand livre, les comptes de classe 9, l'audit inaltérable et consultable |
| **2** | **La TVA opposable** | dériver la TVA du grand livre, l'avoir dans sa période, verrouiller les périodes déclarées, la déductibilité de l'impôt préalable |
| **3** | **Le reste, selon la cible** | bouclement conforme, pilotage, ou le profil privé |

**Les quatre vagues sont matérialisées par quatre jalons GitHub**, et le suivi s'y fait —
pas ici. Ce document ne sera pas mis à jour au fil des corrections : il date du 2026-08-26 et
consigne l'état constaté à cette date.

⚠️ **Grain des issues — consigne du Project Lead** : *« une issue séparée, quitte à en ouvrir
trente. Je préfère beaucoup d'issues qu'on traite l'une après l'autre de manière maîtrisable
qu'un gros truc difficile à gérer et qui prend du temps. »*

---

## XII. Ce que cet audit ne dit pas

Par honnêteté sur sa portée :

- **il n'a pas relu le code ligne à ligne.** Les trois experts ont **balayé large**, sur
  consigne — c'est ainsi qu'a été trouvé l'encaissement manquant, mais un balayage laisse
  passer ce qui ne se voit qu'en profondeur ;
- **il n'a pas testé le logiciel en usage réel.** Toutes les conclusions sont tirées du code,
  des migrations et des manuels ;
- **il ne juge ni la qualité technique, ni la sécurité, ni l'ergonomie** — trois mandats
  comptables et financiers, rien d'autre ;
- **les avis intégraux ne sont pas reproduits ici.** Ce document en consigne la substance et
  les preuves ; il ne les remplace pas.
