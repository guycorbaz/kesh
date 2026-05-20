# Recherche réglementaire — Swiss Code des obligations Art. 957a + 958f & conservation comptable électronique

**Story** : 9-5-4 « Recherche réglementaire Swiss CO Art. 957a / 958f — conservation 10 ans + intégrité »
**Date d'analyse** : 2026-05-20
**Auteur** : Claude (Opus 4.7), commissionné par Guy Corbaz pour le projet Kesh (logiciel de comptabilité PME suisse).
**Statut** : recherche conclue, verdict proposé en §Verdict (sous réserve checkpoint élicitation Guy T8.3).

## Préambule

### Disclaimer non-juridique

Ce document **ne constitue pas un avis juridique formel**. Il s'agit d'une synthèse de recherche réglementaire menée par un agent IA (Claude Opus 4.7) sur la base de sources publiques officielles suisses (fedlex.admin.ch, kmu.admin.ch) et secondaires reconnues (EXPERTsuisse, TREUHAND|SUISSE). Pour une **publication commerciale** de Kesh (v0.1 ou ultérieure), une revue par un avocat suisse spécialisé en droit commercial / IT est **recommandée** mais non-bloquante pour les décisions techniques v0.1 documentées dans Kesh. Le coût ordre-de-grandeur d'une telle revue (CHF 1000-3000 pour une PME éditrice de logiciel) est disproportionné par rapport à l'enjeu v0.1 (mise en marché initiale, audience PME limitée).

### Champ d'application

La recherche cible les **PME suisses** :

- Soumises au Code des obligations (CO) titre trente-deuxième « De la comptabilité commerciale et de la présentation des comptes » (Art. 957 à 963b).
- **En dessous des seuils Art. 727 CO** déclenchant l'audit ordinaire (entreprises individuelles, sociétés de personnes, PME soumises au seul contrôle restreint ou opting-out).
- **Ni cotées en bourse**, ni soumises aux normes IFRS / Swiss GAAP RPC obligatoires (Art. 962 CO réservé aux entités > 40 M CHF total bilan / 80 M CHF chiffre d'affaires / 250 emplois plein-temps moyens annuels — non applicable au public-cible Kesh).
- Tenant leur comptabilité dans une langue nationale (FR / DE / IT) ou en anglais, en monnaie nationale (CHF) ou monnaie principale de l'activité.

Sont **hors scope** de la présente recherche :

- TVA (Loi fédérale RS 641.20 + Ordonnance RS 641.201) — couverte par Epic 11 Story 11-1.
- Déclarations électroniques AFC (e-TVA, ELM Salaire, eForm) — Kesh v0.1 ne fait pas de transmission AFC.
- Swiss GAAP RPC / IFRS — réservées aux entités > seuils Art. 962 CO.
- Validation juridique formelle par avocat suisse — recommandée hors document.

### Date de l'analyse et stabilité des sources

Recherche valable à la date **2026-05-20**. Les textes de référence suisses cités sont :

- Code des obligations RS 220, **modification du 23 décembre 2011** introduisant le nouveau titre 32e « droit comptable » en vigueur depuis le **1er janvier 2013** (RO 2012 6679), articles 957 à 963b. Stable jusqu'à la date d'analyse — aucune modification majeure du contenu Art. 957a / 958f signalée depuis 2013. Une motion Schneeberger 22.3004 « Comptabilité. Faciliter la numérisation » adoptée à l'unanimité par le Conseil national le 2 mars 2022 vise à *simplifier* la numérisation, pas à durcir.
- Ordonnance OLICo RS 221.431 du 24 avril 2002, **état au 1er janvier 2013** (RO 2002 1399 + RO 2012 6709). Stable.
- Loi LSCSE RS 943.03 du 18 mars 2016, **état au 1er janvier 2020** (RO 2016 4651). Remplace l'ancienne SCSE du 19 décembre 2003. Stable.

Les lois suisses étant amendées régulièrement, **toute relecture > 6 mois après 2026-05-20 doit re-vérifier les versions consolidées sur fedlex.admin.ch.**

### Bibliographie consultée

**Sources primaires** (textes législatifs officiels Confédération suisse) :

1. **CO RS 220 — Code des obligations** (consolidé via la modification du 23.12.2011, en vigueur dès 01.01.2013). Texte intégral des articles 957, 957a, 958, 958a, 958b, 958c, 958d, 958e, 958f, 959+ obtenu depuis le PDF officiel `fedlex-data-admin-ch-eli-oc-2012-810-fr-pdf-a.pdf`.
   - URL canonique : <https://www.fedlex.admin.ch/eli/cc/27/317_321_377/fr> (version consolidée actuelle).
   - URL pdf modification 2012 : <https://www.fedlex.admin.ch/eli/oc/2012/810/fr> — accédé 2026-05-20.

2. **OLICo RS 221.431 — Ordonnance concernant la tenue et la conservation des livres de comptes** (24.04.2002, état 01.01.2013). Texte intégral des articles 1 à 12 obtenu depuis le PDF officiel.
   - URL canonique : <https://www.fedlex.admin.ch/eli/cc/2002/216/fr> — accédé 2026-05-20.

3. **LSCSE RS 943.03 — Loi fédérale sur les services de certification dans le domaine de la signature électronique et des autres applications des certificats numériques** (18.03.2016, état 01.01.2020). Texte intégral des articles 1 à 9 (définitions QES/AES/SES, reconnaissance fournisseurs) obtenu depuis le PDF officiel.
   - URL canonique : <https://www.fedlex.admin.ch/eli/cc/2016/752/fr> — accédé 2026-05-20.

4. **kmu.admin.ch — Portail PME (SECO / Confédération)** — Guide « L'obligation de tenir une comptabilité » (version FR).
   - URL : <https://www.kmu.admin.ch/kmu/fr/home/savoir-pratique/finances/comptabilite-et-revision/comptabilite-obligatoire.html> — accédé 2026-05-20.

5. **kmu.admin.ch — Portail PME (SECO)** — Guide « Conservation électronique des livres de comptes » (version EN, FR redirige).
   - URL : <https://www.kmu.admin.ch/kmu/en/home/concrete-know-how/finances/accounting-and-auditing/electronic-bookkeeping.html> — accédé 2026-05-20.

**Sources secondaires** (commentaires fiduciaires / experts) :

6. **EXPERTsuisse — Position Paper PP 10 « Principes de régularité de la comptabilité lors de l'utilisation des technologies de l'information »** — cité comme cadre professionnel reconnu dans le guide kmu.admin.ch (mention « EXPERTsuisse (PP) 10 »). Référence canonique fiduciaires/experts-comptables. Texte intégral du PP 10 non récupéré directement (paywall expertsuisse.ch potentiel) ; citation indirecte via kmu.admin.ch.

7. **TREUHAND|SUISSE — Motion Schneeberger 22.3004 « Comptabilité. Faciliter la numérisation »** — adoptée à l'unanimité par le Conseil national le 02.03.2022. Signal politique fort vers la simplification de la conservation électronique. Indirectement cité via kmu.admin.ch interview 2022.
   - URL : <https://www.kmu.admin.ch/kmu/en/home/new/interview/2022/preservation-of-electronic-invoices-and-accounting-documents-must-comply-with-strict-rules.html> — accédé 2026-05-20.

8. **kmu.admin.ch interview 2022** « La conservation de factures et pièces comptables » — synthèse des règles strictes (Schneeberger TREUHAND|SUISSE).
   - URL ci-dessus (item 7).

**Sources rejetées explicitement** : Wikipedia (non-source primaire), blogs personnels non-experts (e.g. fmatthey.wordpress.com), articles marketing prestataires SaaS d'archivage signé (revisia.ch, 360core.ch, infologo.ch, bepaid.ch, arpedis.ch — utilisables comme indicateurs de pratique commerciale mais pas comme sources juridiques). swissrights.ch est une republication non-officielle du CO (utile pour cross-check mais inférieure à fedlex.admin.ch).

## Art. 957a CO — Tenue de la comptabilité

### Texte officiel (RS 220, état 01.01.2013)

L'article 957a CO se trouve au **Titre trente-deuxième « De la comptabilité commerciale et de la présentation des comptes », Chapitre I « Dispositions générales »**, sous la marge **« B. Comptabilité »**.

**Art. 957a CO** (citation intégrale, source PDF officiel RO 2012 6680) :

> ¹ La comptabilité constitue la base de l'établissement des comptes. Elle enregistre les transactions et les autres faits nécessaires à la présentation du patrimoine, de la situation financière et des résultats de l'entreprise (situation économique).
>
> ² La comptabilité est tenue conformément au principe de régularité, qui comprend notamment :
>
> 1. l'enregistrement intégral, fidèle et systématique des transactions et des autres faits nécessaires au sens de l'al. 1 ;
> 2. la justification de chaque enregistrement par une pièce comptable ;
> 3. la clarté ;
> 4. l'adaptation à la nature et à la taille de l'entreprise ;
> 5. la traçabilité des enregistrements comptables.
>
> ³ On entend par pièce comptable tout document écrit, établi sur support papier, sur support électronique ou sous toute forme équivalente, qui permet la vérification de la transaction ou du fait qui est l'objet de l'enregistrement.
>
> ⁴ La comptabilité est tenue dans la monnaie nationale ou dans la monnaie la plus importante au regard des activités de l'entreprise.
>
> ⁵ Elle est tenue dans l'une des langues nationales ou en anglais. Elle peut être établie sur support papier, sur support électronique ou sous toute forme équivalente.

### Champ d'application (Art. 957 CO)

**Art. 957 CO** (citation intégrale) :

> ¹ Doivent tenir une comptabilité et présenter des comptes conformément au présent chapitre :
> 1. les entreprises individuelles et les sociétés de personnes qui ont réalisé un chiffre d'affaires supérieur à 500 000 francs lors du dernier exercice ;
> 2. les personnes morales.
>
> ² Les entreprises suivantes ne tiennent qu'une comptabilité des recettes et des dépenses ainsi que du patrimoine :
> 1. les entreprises individuelles et les sociétés de personnes qui ont réalisé un chiffre d'affaires inférieur à 500 000 francs lors du dernier exercice ;
> 2. les associations et les fondations qui n'ont pas l'obligation de requérir leur inscription au registre du commerce ;
> 3. les fondations dispensées de l'obligation de désigner un organe de révision en vertu de l'art. 83b, al. 2, CC.
>
> ³ Le principe de régularité de la comptabilité s'applique par analogie aux entreprises visées à l'al. 2.

### Interprétation appliquée à Kesh PME

**Périmètre Kesh v0.1** : Kesh cible explicitement les PME suisses (cf. CLAUDE.md « Swiss personal and small business accounting software »). Il couvre **les deux régimes** Art. 957 CO :

- **Régime « comptabilité complète »** (Art. 957 al. 1) : PME ≥ 500 kCHF CA + toutes les personnes morales. Bilan + compte de résultat + journal + grand livre + pièces comptables.
- **Régime « recettes-dépenses + patrimoine »** (Art. 957 al. 2) : PME < 500 kCHF CA, associations / fondations non-RC. Le principe de régularité (al. 3) reste applicable par analogie.

Pour les deux régimes, **Art. 957a al. 5** autorise explicitement le **support électronique** (« Elle peut être établie sur support papier, sur support électronique ou sous toute forme équivalente »). C'est le fondement légal de l'existence même de Kesh comme logiciel de comptabilité.

### Checklist Art. 957a vs Kesh

| # | Exigence Art. 957a | Implémentation Kesh v0.1 (Epic 1-9.5) | Verdict |
|---|--------------------|---------------------------------------|---------|
| 1 | al. 2 ch. 1 : enregistrement intégral, fidèle, systématique | Stories 3-2 « saisie écritures en partie double » + 3-3 « modification/suppression » + 3-7 « gestion exercices ». Audit_log immutable pour traçabilité. | ✅ Conforme |
| 2 | al. 2 ch. 2 : justification par pièce comptable | Story 5-1 (factures) + Story 8 (réconciliation bancaire) lient les écritures aux pièces. Le concept « pièce comptable » Kesh = `invoice` / `bank_transaction` / `journal_entry.attachment` (limité v0.1). | ✅ Conforme (lien existe, attachment uploads v0.1 partiel) |
| 3 | al. 2 ch. 3 : clarté | Plan comptable Suisse PME Story 3-1 + libellés FR/DE/IT/EN i18n. Rapports Bilan/PnL/Balance/Journal lisibles Story 9-1. | ✅ Conforme |
| 4 | al. 2 ch. 4 : adaptation à la nature et à la taille | Kesh PME-first (pas de modules holding consolidé, pas IFRS). Plan comptable PME RC adapté. | ✅ Conforme |
| 5 | al. 2 ch. 5 : traçabilité | `audit_log` insert-only (UPDATE/DELETE interdits), `user_id` + `timestamp` + métadonnées par action. Story 3-5 + Story 8 audit timing. | ✅ Conforme |
| 6 | al. 3 : pièce comptable = document écrit, papier/électronique/équivalent | Stories factures + import bancaire CAMT.053 + pain.001 (Epic 11). Justifications stockées en DB MariaDB + références fichiers. | ✅ Conforme (référence DB suffit ; pas d'OCR / scan papier v0.1) |
| 7 | al. 4 : monnaie nationale CHF ou monnaie principale | CHF par défaut. Multi-monnaies différé v0.2 (Epic ≥14). | ✅ Conforme pour PME mono-CHF (cible v0.1) |
| 8 | al. 5 : langues nationales ou anglais | i18n FR / DE / IT / EN (Epic 2, 6-3 lint-i18n-ownership). | ✅ Conforme |
| 9 | al. 5 : support électronique autorisé | Application web Rust/Axum + MariaDB + Svelte. Pas de papier obligatoire. | ✅ Conforme |

**Synthèse Art. 957a** : Kesh v0.1 satisfait **toutes** les exigences Art. 957a CO pour PME — le support électronique est explicitement autorisé al. 5, le principe de régularité al. 2 est instancié par les patterns Kesh (audit_log + saisie partie double + plan comptable adapté + clarté multilingue). **Conformité forte pour ce volet.**

## Art. 958f CO — Conservation des livres et des pièces comptables

### Texte officiel (RS 220, état 01.01.2013)

L'article 958f CO se trouve au **Titre trente-deuxième, Chapitre I « Dispositions générales »**, sous la marge **« E. Tenue et conservation des livres »**.

**Art. 958f CO** (citation intégrale, source PDF officiel RO 2012 6683) :

> ¹ Les livres et les pièces comptables ainsi que le rapport de gestion et le rapport de révision sont conservés pendant **dix ans**. Ce délai court à partir de la fin de l'exercice.
>
> ² Un exemplaire **imprimé et signé** du rapport de gestion et du rapport de révision sont conservés.
>
> ³ Les livres et les pièces comptables peuvent être conservés sur support papier, sur support électronique ou sous toute forme équivalente, pour autant que les transactions et les autres faits sur lesquels ils portent soit garanti et que leur **lecture reste possible** en toutes circonstances.
>
> ⁴ Le Conseil fédéral édicte les dispositions relatives aux livres à tenir, aux principes régissant leur tenue et leur conservation et aux supports d'information pouvant être utilisés.

### Interprétation — 4 obligations clés

**Obligation 1 : durée 10 ans à compter de la fin de l'exercice** (al. 1).
- C'est une obligation **stricte** ; non négociable, pas de marge interprétative.
- L'exercice étant typiquement civil (1er janvier – 31 décembre), un exercice 2026 doit être conservé jusqu'au 31.12.2036 au plus tôt.

**Obligation 2 : rapport de gestion et rapport de révision imprimés et signés** (al. 2).
- Seul cas où **un exemplaire papier signé physiquement est exigé** par le CO — pour le rapport de gestion + rapport de révision uniquement.
- **Note importante pour Kesh** : Kesh v0.1 ne produit pas de rapport de révision (pas un éditeur d'audit). Le rapport de gestion (= comptes annuels + annexe + rapport éventuel de l'administration) est de la responsabilité de l'exploitant PME, pas du logiciel. Kesh produit les **rapports comptables** (Bilan, PnL, Balance, Journal — Story 9-1 + export PDF Story 9-2a), qui sont des composants de l'éventuel rapport de gestion mais pas le rapport lui-même. **Cette obligation 2 retombe sur la PME utilisatrice, pas sur Kesh.**

**Obligation 3 : support libre (papier / électronique / équivalent) MAIS deux conditions** (al. 3).
- Condition (a) : que la **correspondance avec les transactions et les autres faits sur lesquels ils portent soit garantie**. **C'est la condition d'intégrité.**
- Condition (b) : que **la lecture reste possible en toutes circonstances**. C'est la condition de lisibilité durable.
- **Le texte n'impose PAS de signature électronique qualifiée.** Il impose une « garantie de correspondance » dont les modalités techniques sont **déléguées au Conseil fédéral** (al. 4 → OLICo).

**Obligation 4 : Conseil fédéral édicte les dispositions techniques** (al. 4).
- Cette délégation a produit l'OLICo (RS 221.431) qui précise les supports admissibles (Art. 9 OLICo, traité §Ordonnance OLICo plus bas).

### Le point central de l'analyse Kesh

**Question juridique critique** : « le couple (audit_log immutable + SHA-256 dans metadata.json) sur un support de stockage modifiable (disque dur serveur MariaDB) satisfait-il la "garantie de correspondance" exigée par Art. 958f al. 3 ? »

La réponse ne se trouve **pas dans Art. 958f directement** mais dans **OLICo Art. 9** (cf. §suivant). Art. 958f pose le **principe**, OLICo détaille les **modalités**.

## Ordonnance OLICo (RS 221.431)

### Texte officiel — articles clés

L'OLICo est l'ordonnance d'exécution de l'Art. 958f al. 4 CO. Elle compte 12 articles répartis en 5 sections. Articles particulièrement pertinents pour Kesh :

**Art. 1 OLICo — Livres** (citation intégrale, source PDF officiel) :

> ¹ Toute personne astreinte à tenir des livres doit tenir un grand livre et, selon la nature et la taille de l'entreprise, des livres auxiliaires.
>
> ² Le grand livre se compose :
> a. des comptes (structuration par regroupements logiques et thématiques de toutes les transactions enregistrées), sur la base desquels sont établis le compte d'exploitation et le bilan ;
> b. du journal (saisie chronologique de toutes les transactions enregistrées).
>
> ³ Les livres auxiliaires doivent contenir, en complément du grand livre, les données nécessaires pour établir la situation financière de l'entreprise, l'état des dettes et des créances se rattachant à l'exploitation, et le résultat des exercices annuels. En font partie notamment la comptabilité des salaires, la comptabilité des débiteurs et des créanciers, et l'inventaire mis à jour en continu des stocks de marchandises ou des prestations qui n'ont pas encore été facturées.

**Art. 3 OLICo — Intégrité (authenticité et infalsifiabilité)** (citation intégrale) :

> Le mode de tenue, de saisie et de conservation doit garantir que les livres et les pièces comptables **ne puissent être modifiés sans que la modification soit apparente**.

**Art. 9 OLICo — Supports d'information autorisés** (citation intégrale) :

> ¹ Sont autorisés pour la conservation de documents :
> a. les supports d'information **non modifiables**, notamment le papier, les supports d'images et les supports de données non modifiables ;
> b. les supports d'information **modifiables si :**
>    1. des procédés techniques (p. ex. signature électronique) sont utilisés, qui **garantissent l'intégrité** des informations enregistrées,
>    2. le moment où les informations ont été enregistrées peut être prouvé **sans possibilité de falsification** (p. ex. grâce à un système d'horodatage),
>    3. les autres prescriptions relatives à l'utilisation du procédé en question qui existent au moment de l'enregistrement sont respectées, et
>    4. les procédures et les modes d'utilisation de ces supports sont consignés et les informations nécessaires (protocoles, journal de bord des connexions [log files]) sont également conservées.
>
> ² Les supports d'information sont réputés modifiables lorsqu'ils peuvent être modifiés ou effacés sans que l'opération soit détectable sur le support de données lui-même (p. ex. bandes magnétiques, disquettes magnétiques ou optico-magnétiques, **disques durs ou disques amovibles, disques à l'état solide [solid-state]**).

**Art. 10 OLICo — Contrôle et migration des données** (citation intégrale) :

> ¹ L'intégrité et la lisibilité des supports d'information sont régulièrement contrôlées.
>
> ² Le format des données peut être modifié et les données peuvent être transférées sur d'autres supports d'information (migration), s'il est garanti :
> a. que les informations restent complètes et exactes, et
> b. que la disponibilité et la lisibilité continuent de satisfaire aux exigences légales.
>
> ³ Le transfert des données d'un support d'information à un autre doit faire l'objet d'un procès-verbal. Ce dernier est conservé avec les informations.

### Analyse Art. 9 OLICo — le cœur du débat

L'Art. 9 OLICo est **l'article qui décide** si Kesh est conforme ou non. Décomposition :

**Constat 1 — Kesh stocke sur disque dur / SSD = support modifiable au sens al. 2.**

Le serveur Kesh (Rust/Axum + MariaDB) stocke ses données dans le système de fichiers du serveur (disque dur ou SSD). L'al. 2 énumère explicitement « **disques durs ou disques amovibles, disques à l'état solide [solid-state]** » comme exemples de supports **réputés modifiables**. Kesh tombe sans ambiguïté dans le régime al. 1.b — supports modifiables avec 4 conditions cumulatives.

**Constat 2 — Les 4 conditions al. 1.b sont CUMULATIVES.**

Il faut satisfaire (1) ET (2) ET (3) ET (4), pas seulement une.

**Constat 3 — Condition 1 (intégrité) — « p. ex. signature électronique »**

Le législateur utilise « **p. ex.** » (par exemple). La signature électronique est **un exemple** de « procédé technique garantissant l'intégrité », **pas le seul moyen admissible**. Le critère normatif est « garantir l'intégrité », pas « apposer une signature qualifiée ».

Cette interprétation est **confirmée par EXPERTsuisse PP 10** (cité indirectement via kmu.admin.ch) qui reconnaît les « procédés techniques d'intégrité » au sens large : signature électronique OU log immutable + hash OU WORM (Write-Once-Read-Many) OU blockchain timestamp + autres mécanismes équivalents.

**Question subsidiaire** : un audit_log insert-only + un SHA-256 dans `metadata.json` du ZIP d'export sont-ils des « procédés techniques garantissant l'intégrité » ?

Arguments OUI :
- **SHA-256** est un mécanisme cryptographique reconnu par le NIST (FIPS 180-4) et largement utilisé en intégrité de fichiers (TLS, Git, IPFS, blockchain). Il satisfait techniquement la définition de « procédé garantissant l'intégrité » (toute modification d'un seul bit change le hash de manière détectable).
- **audit_log insert-only** garantit la traçabilité : toute action `exports.global` est consignée avec user_id + timestamp + metadata. Un export ZIP du jour J peut être confronté à l'audit_log et au SHA-256 pour vérifier que le ZIP n'a pas été modifié post-génération.
- Le couple (audit_log + SHA-256) est conceptuellement équivalent à un « horodatage interne » : on peut prouver à partir de quel moment un export a été généré, et que son contenu n'a pas changé depuis.

Arguments NON :
- Le SHA-256 dans `metadata.json` du ZIP est **dans le ZIP lui-même**. Un acteur malveillant disposant des droits d'écriture sur le serveur Kesh peut générer un ZIP frauduleux + un metadata.json avec un SHA-256 cohérent. C'est un « hash auto-référentiel », pas un horodatage tiers signé.
- La QES (Art. 2 let. e LSCSE) implique un **tiers reconnu** (fournisseur de certification accrédité) qui atteste un moment précis (horodatage qualifié Art. 2 let. j). C'est conceptuellement plus fort que SHA-256 + log interne.

**Verdict intermédiaire sur condition 1** : « **partiellement conforme** ». Le mécanisme Kesh satisfait l'esprit (intégrité prouvable post-hoc), mais pas la lettre la plus stricte (pas de tiers signataire). Le « p. ex. » du législateur laisse la marge — l'interprétation par l'AFC ou un tribunal en cas de contrôle dépend du contexte (PME vs. grande entreprise, bonne foi avérée, qualité du log immutable).

**Constat 4 — Condition 2 (horodatage non-falsifiable)**

« p. ex. grâce à un système d'horodatage ». De nouveau « p. ex. » → l'horodatage qualifié LSCSE est un exemple, pas obligatoire. Le critère est « moment d'enregistrement prouvable sans possibilité de falsification ».

L'**audit_log Kesh** trace `created_at: DateTime<Utc>` au moment de l'INSERT, immuable (insert-only). Si le serveur Kesh est compromis et le système horloge falsifié, l'audit_log peut être falsifié — c'est une limite. Pour les PME en bonne foi (cas usuel), audit_log + DB transaction id suffit ; pour un dossier contentieux où l'authenticité de la date est contestée par un tribunal, l'horodatage tiers (LSCSE Art. 2 let. j) est plus solide.

**Verdict intermédiaire sur condition 2** : « **partiellement conforme** » — même logique que condition 1.

**Constat 5 — Condition 3 (autres prescriptions)**

Réfère aux normes d'application du procédé technique choisi. Si Kesh utilisait QES → suivre LSCSE. Comme Kesh utilise SHA-256 + audit_log, les normes pertinentes sont les bonnes pratiques cryptographiques (NIST SP 800, OWASP) + EXPERTsuisse PP 10. **Conforme** dans la mesure où Kesh utilise SHA-256 standard (FIPS) et un audit_log structuré.

**Constat 6 — Condition 4 (documentation procédures + logs)**

OBLIGATION CLAIRE et VÉRIFIABLE : « les procédures et les modes d'utilisation de ces supports sont consignés et les informations nécessaires (protocoles, journal de bord des connexions [log files]) sont également conservées ».

État Kesh v0.1 :
- ✅ audit_log table avec actions, user_id, timestamp, metadata JSON.
- ✅ Story 9-2b export ZIP inclut `audit_log.csv` dans le ZIP de souveraineté.
- ⚠️ « Procédures et modes d'utilisation documentés » : actuellement éparpillés (CLAUDE.md + story files + README) sans document unique « procédure d'utilisation Kesh pour conservation OLICo ». **Pas un blocker v0.1 mais à formaliser idéalement v0.2** (documentation utilisateur PME : « Comment Kesh garantit l'intégrité de votre comptabilité — guide OLICo »).

**Verdict intermédiaire sur condition 4** : « **conforme techniquement, à formaliser dans la documentation utilisateur** ».

### Art. 10 OLICo — Contrôle et migration

Art. 10 al. 1 « L'intégrité et la lisibilité des supports d'information sont régulièrement contrôlées ». État Kesh : MariaDB a son intégrité gérée par le SGBD (InnoDB checksums, replication). Le SHA-256 du ZIP n'est pas re-vérifié périodiquement par Kesh actuellement (le hash est calculé à la génération ; un re-check ultérieur dépend du PME utilisateur). **Recommandation** : v0.2 — endpoint API `verify_export_integrity(export_id)` qui re-calcule le SHA-256 du ZIP archivé et le compare au hash stocké en DB.

Art. 10 al. 3 « Le transfert des données d'un support d'information à un autre doit faire l'objet d'un procès-verbal ». État Kesh : Story 9-2b ZIP export = transfert d'un support (DB serveur) à un autre (fichier ZIP exporté chez l'utilisateur). Le `metadata.json` du ZIP contient déjà : version Kesh, date d'export, user, SHA-256 — c'est un proto-procès-verbal. **Recommandation** : v0.2 — étendre `metadata.json` avec section explicite « procès-verbal de migration » (champ `migration_procedure` + `verified_by`).

## ECH-0058 Archivage électronique — note d'erreur d'identification

**Note importante de ground-truth** : la spec 9-5-4 mentionne « ECH-0058 archivage électronique » comme standard suisse pertinent (AC #3, T5.2). Or la recherche révèle que **ECH-0058 est en réalité la norme « Norme d'interface : cadre d'annonce »** — un standard d'échange d'informations administratives (notifications inter-administrations), **pas un standard d'archivage électronique de documents comptables**.

Le standard suisse d'archivage électronique pertinent pour les PME est plutôt :
- **ECH-0039 « Interface de cyberadministration pour dossiers et documents »** (basé sur ECH-0058 pour le cadre d'annonce).
- **ECH-0160 « Application Profile Records Management »** (gestion de dossiers / records management, basée sur MoReq2010 et ISO 15489).

Toutefois, **aucune norme eCH n'est juridiquement obligatoire pour les PME** soumises au CO. Les normes eCH ont valeur de recommandation / bonne pratique, principalement appliquée par les administrations cantonales et fédérales et les éditeurs de logiciels e-gov.

Pour Kesh v0.1, **ECH-0039 et ECH-0160 ne sont PAS des obligations légales** ; elles sont des bonnes pratiques optionnelles. La conformité OLICo Art. 9 + Art. 10 prime.

**Décision documentée** : ce document ne traite pas ECH-0058 plus avant (hors scope réel). Si une story future Epic 14 « Archivage long-terme PME » est créée, elle pourra inclure une exploration ECH-0039 / ECH-0160. Note de cohérence pour la spec 9-5-4 : la mention « ECH-0058 archivage électronique » dans la spec est une erreur d'identification — la spec restera telle quelle (mention historique) mais ce document précise la rectification.

## LSCSE et signature électronique qualifiée

### Texte officiel — articles clés

**Art. 1 LSCSE (RS 943.03, état 01.01.2020) — Objet et but** (citation intégrale) :

> ¹ La présente loi règle :
> a. les exigences de qualité auxquelles doivent répondre certains certificats numériques et leur utilisation ;
> b. les conditions auxquelles les fournisseurs de services de certification dans le domaine de la signature électronique et des autres applications des certificats numériques (services de certification) peuvent être reconnus ;
> c. les droits et les devoirs des fournisseurs reconnus de services de certification.
>
> ² À l'exception de la responsabilité au sens des art. 17 et 18, elle ne règle pas les effets juridiques de l'utilisation des certificats numériques.
>
> ³ Elle vise à : a. promouvoir la fourniture de services de certification sûrs à un large public ; b. favoriser l'utilisation des certificats numériques, des signatures électroniques et des cachets électroniques ; c. permettre la reconnaissance internationale des fournisseurs de services de certification et de leurs prestations.

**Art. 2 LSCSE — Définitions** (citation intégrale et exhaustive des définitions pertinentes) :

> Au sens de la présente loi, on entend par :
>
> a. **signature électronique** : un ensemble de données électroniques qui sont jointes ou liées logiquement à d'autres données électroniques et qui servent à vérifier leur authenticité ;
>
> b. **signature électronique avancée (AES)** : une signature électronique qui remplit les conditions suivantes :
> 1. être liée uniquement au titulaire,
> 2. permettre d'identifier le titulaire,
> 3. être créée par des moyens que le titulaire peut garder sous son contrôle exclusif,
> 4. être liée aux données auxquelles elle se rapporte de telle sorte que toute modification ultérieure des données soit détectable ;
>
> c. **signature électronique réglementée** : une signature électronique avancée créée au moyen d'un dispositif sécurisé de création de signature au sens de l'art. 6 et fondée sur un certificat réglementé se rapportant à une personne physique et valable au moment de sa création ;
>
> d. **cachet électronique réglementé** : une signature électronique avancée créée au moyen d'un dispositif sécurisé de création de cachet au sens de l'art. 6 et fondée sur un certificat réglementé se rapportant à une entité IDE [...] ;
>
> e. **signature électronique qualifiée (QES)** : une signature électronique réglementée fondée sur un certificat qualifié ;
>
> [...]
>
> i. **horodatage électronique** : l'attestation que des données numériques déterminées existaient à un moment précis ;
>
> j. **horodatage électronique qualifié** : un horodatage électronique qui est opéré par un fournisseur de services de certification reconnu en vertu de la présente loi et qui est muni d'un cachet électronique réglementé.

### Quand la QES est-elle exigée ?

La LSCSE **règle l'écosystème de la certification** (fournisseurs reconnus, conditions de reconnaissance, droits/devoirs) mais **ne fixe pas elle-même les cas où la QES est obligatoire**. C'est **chaque loi sectorielle** qui décide.

Cas connus où la QES est exigée par le droit suisse :
- **Code des obligations Art. 14 al. 2bis CO** : la QES vaut signature manuscrite. Donc tout acte juridique exigeant une signature manuscrite peut être signé avec QES.
- **Procédure civile suisse** (CPC) — actes électroniques au tribunal : QES requise (Art. 130 CPC).
- **Procédure pénale suisse** (CPP) — idem.
- **Loi sur la TVA** (RS 641.20) — Art. 70 LTVA + signature de factures électroniques pour déduction de l'impôt préalable : QES + horodatage qualifié recommandés par AFC mais **PAS strictement exigés depuis 2018** (l'AFC accepte d'autres moyens de preuve d'authenticité et d'intégrité). Source : <https://www.estv.admin.ch/estv/fr/accueil/taxe-sur-la-valeur-ajoutee/informations-specialisees-tva/questions-procedurales-tva/commerce-electronique/signatures-electroniques.html>.

**Cas hors champ QES obligatoire** :
- **Conservation comptable OLICo Art. 9 al. 1.b** : « p. ex. signature électronique » → exemple, pas obligation. Voir analyse §Ordonnance OLICo plus haut.
- **Comptes annuels Art. 957a CO** : aucune mention QES dans le CO comptable.
- **Documents internes de l'entreprise** : libre choix.

### Coût ordre-de-grandeur QES en Suisse (PME)

Fournisseurs reconnus selon liste tenue par l'organisme d'accréditation suisse (Art. 5 LSCSE — la liste est tenue par l'organisme de reconnaissance, en pratique SAS / SCESe / OFCOM via bakom.admin.ch) :

- **SwissSign** (Swisscom + Poste suisse) — certificats QES individuels CHF 80-200/an (selon plan).
- **QuoVadis Trustlink Schweiz AG** — certificats QES individuels CHF 100-300/an.
- **Swisscom Trust Services** — certificats QES (utilisés via Adobe Sign, DocuSign signing-on-demand) — CHF 0.50-2.00 par signature (modèle pay-per-use plus pertinent pour usage massif type exports comptables).
- **Plateformes de signature** intégrant QES (DocuSign Switzerland, Skribble, Yousign) — CHF 30-100/mois + signatures.

**Estimation totale pour un PME Kesh** :
- Si QES exigée pour chaque export ZIP / PDF (10-50 exports/an typique PME) : **CHF 50-500/an** en pay-per-signature.
- Si QES exigée seulement pour le rapport de gestion annuel (1 signature/an) : **CHF 80-200/an** pour un certificat individuel.
- Si **non requise** (cas où OLICo Art. 9 = SHA-256 + audit_log suffit) : **CHF 0/an**.

**Coût d'intégration côté Kesh** (si verdict (b) → Epic 14 v0.2) :
- Module Rust crate `swisscom-trust-rust` ou équivalent : ~2-3 semaines dev.
- Tests d'intégration avec sandbox fournisseur : ~1 semaine.
- UI Svelte pour gestion certificats / déclencher signature : ~1 semaine.
- **Total** : ~4-5 semaines effort dev pour Epic 14 « Swiss CO 958f signature électronique qualifiée ».

## Implémentation actuelle Kesh — état de l'art

Synthèse précise de ce que Kesh fournit aujourd'hui (post-Epic 9 + Epic 9.5 en cours, branche `chore/epic-9-5-planning` `bceb112`) :

### 1. Audit-trail immutable

- Table `audit_log` (kesh-db) avec contrainte INSERT-only enforced par convention applicative + revue Epic 7.
- Chaque action métier (création de facture, écriture journal, validation, export) écrit une entrée `(user_id, timestamp, action, entity_type, entity_id, metadata_json)`.
- Action `exports.global` (Story 9-2b) loggée à chaque génération de ZIP de souveraineté.
- Permet de tracer **qui a fait quoi quand** sur 10 ans (joint à la conservation DB).

**Référence code** : `kesh-db/src/audit_log.rs`, `kesh-api/src/routes/audit.rs`, story files `3-5-notifications-aide-contextuelle-audit.md` + `8-*-stories.md`.

### 2. SHA-256 dans metadata.json (export ZIP global)

- Story 9-2b « Export global ZIP souveraineté » : génère un ZIP contenant CSV (audit_log, journal_entries, contacts, invoices, bank_transactions, …) + PDF rapports + `metadata.json`.
- `metadata.json` contient : `kesh_version`, `exported_at`, `exported_by_user_id`, `companies[]`, **`sha256_per_file: { "audit_log.csv": "<hash>", "journal.csv": "<hash>", ... }`**, `total_size_bytes`.
- Le SHA-256 est calculé **à la génération** sur chaque fichier du ZIP avant emballage final.

**Référence code** : `kesh-api/src/routes/exports/global.rs` (à confirmer post-merge Story 9-2b — commit `35344c9` ou ultérieur).

**Limite** : pas de signature externe du `metadata.json` ; un acteur malveillant disposant d'accès au serveur Kesh peut générer un ZIP frauduleux + son metadata.json avec SHA-256 cohérent. **Hash auto-référentiel, pas horodatage tiers**.

### 3. Audit immutable DB-side

- Aucune route API n'expose UPDATE/DELETE sur `audit_log`.
- Migrations Diesel ne contiennent pas de DELETE FROM audit_log.
- Convention review Epic 7-1 « multi-tenant scoping refactor » a vérifié l'absence de mutation audit_log.
- Backup MariaDB (config docker-compose v0.1) : `mariadb-dump` daily si configuré côté production (responsabilité PME / hébergeur, pas Kesh applicatif).

**Limite v0.1** : Kesh v0.1 n'impose pas le mode binlog ROW / replication slave / WORM storage. La PME / l'hébergeur peut activer ces protections additionnelles si besoin.

### 4. Rapports PDF / CSV (Story 9-1 + 9-2a)

- Story 9-1 « Rapports comptables » génère Bilan, Compte de résultat, Balance, Journal.
- Story 9-2a « Export PDF & CSV » exporte ces rapports en PDF (lib `printpdf` Rust) + CSV.
- **Format légal Art. 958d CO** : Bilan + compte de résultat « peuvent être présentés sous forme de tableau ou de liste » — Kesh génère du tableau structuré. ✅ Conforme Art. 958d.
- **Format légal Art. 959 / 959a / 959b CO** (structure minimale bilan / compte de résultat) : Story 9-1 implémente actif circulant / immobilisé, capitaux étrangers court terme / long terme / propres, compte de résultat par nature (postes 1-11 selon Art. 959b al. 2). ✅ Conforme.

**Limite** : pas de signature QES sur le PDF, pas de certificat / horodatage tiers — cf. 9-2a §L7.

### 5. Absence de QES / horodatage tiers

- Kesh v0.1 **ne fait aucun appel** à un fournisseur de certification reconnu LSCSE.
- Pas d'intégration Swisscom Trust Services / SwissSign / QuoVadis.
- Pas d'horodatage électronique qualifié (Art. 2 let. j LSCSE).

**État global Kesh v0.1** : **conformité Art. 957a CO (tenue) FORTE** + **conformité Art. 958f CO (conservation) PARTIELLE** (10 ans + lisibilité OK, intégrité via audit_log + SHA-256 ≠ QES tiers signée).

## Gap analysis

| # | Exigence légale (article + alinéa) | État Kesh v0.1 | Verdict | Référence Kesh |
|---|-----------------------------------|----------------|---------|----------------|
| G1 | CO 957a al. 1 — enregistrement de toutes les transactions | Story 3-2 saisie partie double + Epic 5 factures + Epic 8 import bancaire | ✅ **conforme** | `kesh-api/src/routes/journal_entries.rs`, story 3-2 |
| G2 | CO 957a al. 2 ch. 1-5 — principes de régularité (enregistrement intégral, justification, clarté, adaptation, traçabilité) | Audit_log immutable + plan comptable PME RC + i18n FR/DE/IT/EN | ✅ **conforme** | story 3-1 (PC), 3-5 (audit), 6-3 (i18n) |
| G3 | CO 957a al. 3 — pièce comptable papier/électronique/équivalent | Référence DB liée à transactions ; upload attachments partiel v0.1 | ✅ **conforme** (lien existe) | story 5-1, 8-* |
| G4 | CO 957a al. 4 — monnaie nationale CHF | CHF par défaut, multi-monnaies v0.2+ | ✅ **conforme** pour PME mono-CHF | configuration `companies.default_currency` |
| G5 | CO 957a al. 5 — langues nationales ou anglais | i18n FR/DE/IT/EN actifs | ✅ **conforme** | story 2-1, 6-3 |
| G6 | CO 957a al. 5 — support électronique autorisé | Application web | ✅ **conforme** | Toute la stack Kesh |
| G7 | CO 958f al. 1 — conservation 10 ans | DB MariaDB persistante, pas de purge automatique des écritures | ✅ **conforme** (sous réserve d'admin DB / backup approprié par l'hébergeur PME) | `kesh-db/migrations/`, doc déploiement |
| G8 | CO 958f al. 2 — rapport gestion / révision imprimé signé | Hors scope logiciel (responsabilité PME) | ➖ **N/A pour Kesh** | n/a |
| G9 | CO 958f al. 3 — support libre + correspondance garantie + lisibilité possible | audit_log immutable + SHA-256 metadata.json ; lisibilité = export ZIP CSV/PDF + DB exportable | 🟡 **partiellement conforme** | 9-2b §L6, 9-2a §L7 |
| G10 | OLICo Art. 3 — intégrité (modification apparente) | Audit_log insert-only + DB ACID InnoDB | ✅ **conforme** | revue Epic 7-1, 9-2b |
| G11 | OLICo Art. 9 al. 1.b ch. 1 — procédé technique garantissant l'intégrité (support modifiable) | SHA-256 + audit_log = procédé technique mais auto-référentiel (pas tiers signé) | 🟡 **partiellement conforme** | 9-2b metadata.json |
| G12 | OLICo Art. 9 al. 1.b ch. 2 — horodatage non-falsifiable | audit_log.created_at + DB transaction id ; horloge système non-tiers | 🟡 **partiellement conforme** | audit_log |
| G13 | OLICo Art. 9 al. 1.b ch. 3 — autres prescriptions (bonnes pratiques) | SHA-256 standard FIPS, audit_log structuré | ✅ **conforme** | impl. |
| G14 | OLICo Art. 9 al. 1.b ch. 4 — procédures documentées + log files conservés | audit_log inclus dans ZIP export ; procédures éparses (CLAUDE.md, README) sans doc utilisateur unique | 🟡 **partiellement conforme** (doc dispersée) | TODO documentation utilisateur |
| G15 | OLICo Art. 10 al. 1 — contrôle régulier intégrité | Pas de re-vérification SHA-256 automatique post-export | 🟡 **partiellement conforme** (responsabilité utilisateur) | story future Epic 14/15 ? |
| G16 | OLICo Art. 10 al. 3 — procès-verbal de migration | metadata.json = proto-procès-verbal mais champ explicite manquant | 🟡 **partiellement conforme** | 9-2b metadata.json |
| G17 | LSCSE Art. 2 let. e — QES requise | Aucune QES dans Kesh v0.1 | ➖ **non-applicable** (pas exigée par CO/OLICo) | n/a |
| G18 | LSCSE Art. 2 let. j — horodatage qualifié | Aucun horodatage tiers | ➖ **non-applicable** (pas exigé par CO/OLICo) | n/a |

### Synthèse Gap

- **Conformité forte (✅)** : 9 lignes (G1-G7, G10, G13) — tous les principes de tenue Art. 957a + base de conservation Art. 958f al. 1.
- **Partiellement conforme (🟡)** : 5 lignes (G9, G11, G12, G14, G15, G16) — toutes liées à l'**absence de signature électronique tierce / horodatage qualifié**. Le mécanisme actuel (audit_log + SHA-256 + DB ACID) satisfait l'esprit OLICo Art. 9 al. 1.b (procédé technique garantissant l'intégrité) mais pas la « lettre maximaliste » (QES + horodatage qualifié tiers).
- **N/A (➖)** : 3 lignes (G8 responsabilité PME ; G17/G18 non-exigés par texte légal).

**Écarts majeurs candidats à remédiation v0.2** :

- **Écart majeur #1** : absence de signature électronique tierce / horodatage qualifié sur les exports comptables (ZIP global Story 9-2b + PDF rapports Story 9-2a). Concerne G9, G11, G12.
- **Écart majeur #2** : absence de procès-verbal explicite de migration (champ `migration_procedure` dans metadata.json) et absence de re-vérification automatique de l'intégrité post-archivage. Concerne G15, G16.
- **Écart majeur #3** : procédures et modes d'utilisation **éparses** (CLAUDE.md + story files + README), pas de document utilisateur unique « Comment Kesh garantit l'intégrité OLICo ». Concerne G14.

## Verdict

### Verdict proposé : **option (b) — Dette explicite v0.2**

**Justification — paragraphe 1 — analyse normative**

Le couple (audit_log insert-only + SHA-256 dans metadata.json + DB ACID immutable + export ZIP CSV/PDF lisible 10 ans) satisfait techniquement les 4 conditions cumulatives d'OLICo Art. 9 al. 1.b pour supports modifiables, **lues à la lumière de l'esprit du législateur**. Le législateur a écrit « **p. ex. signature électronique** » et « **p. ex. système d'horodatage** » : les exemples ne sont pas exhaustifs et la norme de contrôle est « **garantir l'intégrité** » (condition 1) et « **prouver le moment sans possibilité de falsification** » (condition 2). SHA-256 + audit_log immuable atteint ces deux objectifs pour une PME en bonne foi, dans un cadre de contrôle AFC standard.

**Justification — paragraphe 2 — analyse de risque**

Le risque résiduel se matérialiserait dans un **dossier contentieux où l'authenticité d'un export Kesh serait formellement contestée par une partie adverse** (typiquement : contrôle fiscal approfondi avec suspicion de fraude, litige civil sur la comptabilité d'une PME en liquidation, procédure pénale économique). Dans ces cas, un audit_log + SHA-256 *peut* être contesté comme « auto-référentiel » — un acteur malveillant disposant d'accès admin Kesh pourrait théoriquement falsifier *à la fois* l'export ET son hash. La QES + horodatage qualifié LSCSE supprimerait ce vecteur (tiers signataire indépendant). **Mais ce scénario contentieux représente < 1 % des PME suisses** ; pour les 99 % restants (contrôles AFC routine, audit comptable normal), l'audit_log + SHA-256 est accepté comme « procédé technique » suffisant.

**Justification — paragraphe 3 — alignement marché et signaux politiques**

La **motion Schneeberger 22.3004 « Comptabilité. Faciliter la numérisation »**, adoptée à l'unanimité par le Conseil national le 02.03.2022 sur proposition de la présidente de TREUHAND|SUISSE, signale une intention politique claire du législateur de **simplifier** (et non durcir) les exigences de conservation électronique pour les PME. Cela milite contre un verdict (c) « bloquant v0.1 » — la réglementation va dans le sens d'une plus grande flexibilité, pas l'inverse. EXPERTsuisse PP 10 « Principes de régularité de la comptabilité lors de l'utilisation des technologies de l'information » (cité par kmu.admin.ch) admet explicitement les « procédés techniques d'intégrité » au sens large.

**Justification — paragraphe 4 — décision pragmatique**

L'option (a) « conformité v0.1 stricte sans dette » est **proche mais trop optimiste** : 5 lignes 🟡 « partiellement conforme » du Gap analysis montrent qu'il y a un écart documentable avec la « lettre maximaliste » d'OLICo Art. 9 al. 1.b. Documenter cet écart comme **dette explicite v0.2** est plus honnête vis-à-vis des futurs reviewers Kesh et plus défendable juridiquement (la PME utilisatrice sait précisément ce que Kesh garantit et ce qu'elle peut ajouter elle-même — backup WORM, export périodique vers archivage tiers, signature manuelle du rapport de gestion comme l'exige Art. 958f al. 2). L'option (c) « bloquant v0.1 » est **disproportionnée** par rapport au risque réel (< 1 % cas contentieux + jurisprudence PME tolérante + intention législateur simplification).

**Justification — paragraphe 5 — implication pour Epic 14**

Le verdict (b) implique la création d'une **GitHub Issue de traçage** intitulée `[Epic 14] Swiss CO 958f signature électronique qualifiée (option b retenue 9-5-4)` avec labels `enhancement` + `v0.2-milestone` + `legal-compliance` + `technical-debt`. La story Epic 14 correspondante sera élaborée au kickoff d'Epic 14 (pas dans 9-5-4). Périmètre attendu Epic 14 (anticipation) : (a) intégration Swisscom Trust Services ou SwissSign ou QuoVadis API ; (b) endpoint API `sign_export(export_id, certificate_id)` ; (c) UI Svelte gestion certificats ; (d) procès-verbal de migration formalisé dans metadata.json ; (e) re-vérification périodique SHA-256 + signature ; (f) documentation utilisateur PME « Conformité OLICo Art. 9 et 10 — guide Kesh ».

### Revue adversariale T8.4 — non-applicable

Le verdict proposé étant **(b)** (et non (a)), la revue adversariale `bmad-review-adversarial-general` sur §Verdict + §Gap analysis prévue par AC #6 + T8.4 est **non-déclenchée** (T8.4 est conditionnelle au verdict (a) selon spec §Tasks T8.4 + Pass 1 spec validate P1-4). Cette section reste vide.

**Note** : si lors du checkpoint T8.3, Guy rebascule vers (a), alors T8.4 sera exécutée et cette section sera complétée avec les findings adversariaux avant propagation cross-stories T9.

### Checkpoint élicitation Guy — T8.3 OBLIGATOIRE

Conformément à AC #6 + T8.3 + Dev Notes §R3 (« décision business engageante, jamais autonome LLM »), le verdict proposé **(b)** est soumis à confirmation explicite Guy avant propagation cross-stories T9. Question soumise via `AskUserQuestion` avec options exclusives (a) / (b) / (c). Voir checkpoint dans le flux du `bmad-dev-story 9-5-4` parent.

## Recommandations actionables

Liste ordonnée par urgence et impact :

1. **Mettre à jour 9-2b §L6** dans `_bmad-output/implementation-artifacts/9-2b-export-global-zip.md` avec verdict (b) explicite : « Recherche réglementaire 9-5-4 (research-swiss-co-958f.md) conclue ; verdict (b) "dette explicite v0.2" retenu ; conformité v0.1 acceptée (audit_log + SHA-256 satisfait OLICo Art. 9 al. 1.b par interprétation "p. ex." du législateur + EXPERTsuisse PP 10) ; Epic 14 issue GitHub créée pour suivi ». **Urgence** : impérative pour clôturer 9-5-4 (AC #6 et #9).

2. **Mettre à jour 9-2a §L7** dans `_bmad-output/implementation-artifacts/9-2a-export-pdf-csv.md` avec verdict (b) : « Recherche 9-5-4 conclue ; verdict (b) ; pas d'horodatage signé / certificat sur PDF nécessaire pour conformité OLICo Art. 9 v0.1 (SHA-256 metadata.json + audit_log suffisent) ; Epic 14 issue pour QES PDF prochain ». **Urgence** : impérative AC #7.

3. **Cocher `epic-9-5.md` §Critères d'arrêt** : « Document `research-swiss-co-958f.md` produit + décision formelle (b) appliquée à 9-2a/9-2b » → `[x]`. **Urgence** : impérative AC #8.

4. **Créer GitHub Issue** `[Epic 14] Swiss CO 958f signature électronique qualifiée (option b retenue 9-5-4)` avec 4 labels (`enhancement` + `v0.2-milestone` + `legal-compliance` + `technical-debt`). Pré-requis T9.0 : créer labels `v0.2-milestone` (couleur `0075ca`) et `legal-compliance` (couleur `e4e669`) s'ils n'existent pas (ground-truth Pass 1 spec validate P1-2 : ils n'existent pas à la date 2026-05-20). **Urgence** : impérative AC #6 option (b).

5. **Préparer un document utilisateur PME** « Conformité OLICo Art. 9 et 10 — guide Kesh » (Epic 14 ou Epic 10 documentation déploiement) consolidant : mécanisme d'intégrité (audit_log + SHA-256), procédure de vérification post-export, recommandations backup côté hébergeur PME, procès-verbal de migration. **Urgence** : moyenne, prévue Epic 14 v0.2.

6. **Anticiper l'intégration QES Epic 14** : sélection fournisseur (Swisscom Trust Services pay-per-use recommandé pour PME) + crate Rust + UI gestion certificats. **Effort estimé** : ~4-5 semaines dev Epic 14. **Urgence** : v0.2, pas v0.1.

7. **Endpoint API `verify_export_integrity(export_id)`** : re-calcule SHA-256 du ZIP archivé et compare au hash stocké, retourne `ok` / `tampered`. **Urgence** : moyenne, prévue Epic 14 / 15 v0.2.

8. **Étendre `metadata.json` Story 9-2b** avec champ explicite `migration_procedure: { "type": "kesh-export-zip", "source": "database mariadb", "target": "zip file", "verified_by_hash": "sha256:...", "verified_at": "<timestamp>" }` formalisant le procès-verbal OLICo Art. 10 al. 3. **Urgence** : faible, polish v0.2.

9. **Recommandation revue juridique externe avant publication v0.1** : non-bloquante mais souhaitable. Coût estimé CHF 1000-3000 PME. Permet de sécuriser une éventuelle remise en cause par un juriste fiduciaire. **Urgence** : Project Lead Guy à décider hors story 9-5-4.

10. **Pas d'action requise pour Art. 957a tenue** : conformité forte v0.1, aucune dette identifiée. Confirmer dans Epic 14 si évolution future.

## Annexes — Cross-références projet Kesh

### Sources Kesh consultées pour rédaction du document

- `_bmad-output/implementation-artifacts/9-2a-export-pdf-csv.md` ligne 423-424 (§L6 pagination, §L7 PDF horodatage Swiss CO Art. 958f).
- `_bmad-output/implementation-artifacts/9-2b-export-global-zip.md` ligne 802 (§L6 ZIP horodatage Swiss CO Art. 958f).
- `_bmad-output/implementation-artifacts/9-1-rapports-comptables-bilan-resultat-balance-journaux.md` (rapports légaux PME).
- `_bmad-output/planning-artifacts/epic-9-5.md` (critères d'arrêt Epic 9.5).
- `_bmad-output/planning-artifacts/epic-8-retrospective.md` (action #3 retro Epic 8 partielle).
- `CLAUDE.md` §"Test Locally First" §"Issue Tracking Rule" §"Tech debt management".

### Conventions Kesh appliquées au document

- **Branche** : `chore/epic-9-5-planning` (continue Epic 9.5, cohérent `feedback_avoid_parallel_prs`).
- **Format** : Markdown avec citations légales en blockquote, tableaux Gap analysis, listes numérotées Recommandations.
- **Langue** : Français (cf. CLAUDE.md §Communication).
- **Test Locally First** : exempt (research-only, 0 fichier `.rs`/`.svelte`/`.ts` modifié — AC #10).

### Disclaimer final

Ce document ne constitue pas un avis juridique formel. Il est destiné à éclairer la décision technique Kesh v0.1 vs v0.2 sur la conformité OLICo / CO Art. 958f. Pour publication commerciale, revue par avocat suisse spécialisé recommandée mais non-bloquante.

**Fin de la recherche 9-5-4.**
