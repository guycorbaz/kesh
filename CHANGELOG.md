# Changelog

Toutes les évolutions notables de **Kesh** sont consignées dans ce fichier.

Le format suit la convention [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/) et la numérotation respecte [Semantic Versioning](https://semver.org/lang/fr/).

Le contenu est rédigé en français à destination des **fiduciaires, PME, indépendants et associations** suisses qui utilisent Kesh. Pour le détail technique commit-par-commit, consulter [l'historique Git](https://github.com/guycorbaz/kesh/commits/main) du projet.

---

## [Non publié]

### Ajouté

- **Un compte de produit par ligne de facture** : chaque ligne d'une facture peut désormais porter **son propre compte de produit**, au lieu que tout le chiffre d'affaires atterrisse sur un compte unique. Une facture mêlant honoraires, prestations de services et marchandises ventile son produit **sur les bons comptes dès la validation**, ce qui rend le compte de résultat exploitable sans reclassement manuel a posteriori. Une ligne qui ne précise rien continue de suivre le **compte de produit par défaut de la société** configuré dans les *Réglages* — le comportement actuel est donc strictement conservé pour toutes les factures existantes. Les **avoirs** ventilent en miroir : la contre-passation extourne exactement les comptes que la facture a crédités, y compris si le compte par défaut de la société a changé entre-temps. La colonne apparaît dans l'export CSV des lignes de facture. (#152, CR #265)
  - **Le sélecteur dans le formulaire de facture arrive à la prochaine étape** — pour l'instant le compte n'est modifiable que par l'API. En attendant, enregistrer une facture depuis l'interface remet les lignes sur le compte par défaut.
  - **Factures déjà validées** : elles conservent le comportement actuel (repli sur le compte par défaut au moment d'un éventuel avoir). Leur reprise fait l'objet d'une mise à jour dédiée.

### Corrigé

- **Message clair quand un compte de produit ne convient pas** : lorsqu'une ligne de facture désigne un compte archivé, qui n'est pas un compte de produit, ou qui n'est pas imputable, Kesh **nomme la ligne et le compte concernés** — et **toutes** les lignes fautives d'un coup, pas seulement la première. Auparavant le message se réduisait à « un ou plusieurs comptes sont archivés ou invalides », sans dire lequel : sur une facture de plusieurs dizaines de lignes, il fallait chercher. Le contrôle a lieu à la saisie **et** à la validation, ce dernier fermant deux angles morts : un compte **changé de type** (par exemple passé de produit à charge) ou **rendu non imputable** entre le brouillon et la validation n'était détecté par rien.
- **Avoir sur une facture dont le compte a été archivé** : l'émission est refusée avec un message indiquant **quelle ligne et quel compte réactiver**, au lieu d'une erreur générique. Poster la contre-passation sur un autre compte laisserait un résidu permanent, invisible au bilan mais faux au compte de résultat — un avoir bloqué vaut mieux qu'un avoir sur le mauvais compte.
- **Facture d'un montant total nul** : sa validation renvoie désormais un message explicite (« renseignez au moins une ligne avec un prix unitaire supérieur à zéro ») au lieu d'échouer sur une erreur technique. Idem pour l'émission d'un avoir sur une telle facture.

## [0.8.0] — 2026-07-26

### Ajouté

- **Report à-nouveau automatique — votre bilan reste juste d'une année à l'autre** : les soldes du **bilan** (actifs, passifs, capitaux propres) se **reportent automatiquement d'un exercice à l'autre**, calculés en temps réel depuis l'origine — comme dans un logiciel comptable moderne (Odoo/Flectra). Ouvrir un nouvel exercice n'affiche plus « zéro à l'actif » : les soldes de clôture de l'exercice précédent apparaissent d'emblée, **sans aucune écriture de clôture ni de report à passer à la main**. Le bilan expose désormais deux lignes de fonds propres distinctes : **« Résultat reporté »** (cumul des résultats des exercices antérieurs ; « Perte reportée » s'il est négatif) et **« Résultat de l'exercice »** (résultat de l'exercice courant, en cours d'année comme en fin d'exercice). L'équation du bilan reste équilibrée à tout instant. Le **compte de résultat** et la **balance de vérification** restent, eux, calculés **par exercice** (mouvement de la période) — un rappel l'indique sur l'écran de la balance de vérification pour éviter toute confusion avec le solde cumulé du bilan. Les exports **CSV** et **PDF** du bilan incluent le résultat reporté. La clôture d'un exercice reste un simple **verrou** (immutabilité + audit, CO art. 957-964), sans écriture générée.
- **Le rôle d'un compte ne dépend plus de son numéro** : un correctif important pour les utilisateurs qui **migrent depuis un autre logiciel**. Auparavant, certains numéros de comptes (2800 « capital », 2979) étaient traités de façon particulière au bilan — au risque de faire **disparaître les capitaux propres d'ouverture** saisis lors d'une reprise de comptabilité. Désormais tous les comptes de passif sont comptés normalement, sans exception liée au numéro.
- **Chaque compte porte désormais un rôle explicite** : la page *Plan comptable* affiche et permet de modifier le **rôle** de chaque compte (créances clients, produit par défaut, dettes fournisseurs, TVA récupérable / due / décompte, capital, autres fonds propres, bénéfice reporté, résultat de l'exercice) ainsi qu'un indicateur **postable**. Kesh ne devine plus jamais la fonction d'un compte à partir de son **numéro** : vous pouvez renuméroter ou restructurer votre plan comptable sans rien casser — le plan comptable suisse est un usage, pas une obligation légale. Les rôles qui n'ont de sens qu'une seule fois (créances clients, TVA due, résultat de l'exercice…) sont **garantis uniques** : si vous attribuez un rôle déjà pris, Kesh refuse la modification **en nommant le compte qui le détient**. Un rôle ne peut être posé que sur le **bon côté du plan** : un rôle de bilan (créances, dettes, TVA, fonds propres…) ne peut pas atterrir sur un compte de charge ou de produit, et le *produit par défaut* exige un compte de produit — Kesh refuse l'attribution incohérente plutôt que de la laisser fausser vos écritures plus tard.
  - **À vérifier après la mise à jour** : les rôles ont été **pré-attribués d'après les numéros du plan comptable standard** (1100 → créances clients, 2200 → TVA due, etc.). Si vous aviez **renuméroté** vos comptes, contrôlez la colonne *Rôle* de votre plan comptable et corrigez les attributions.
  - L'indicateur **postable** **bloque désormais la saisie manuelle** : Kesh refuse une écriture saisie à la main (création *ou* modification) dont une ligne vise un compte non-postable — typiquement un compte de regroupement ou le compte de résultat. Les écritures manuelles **déjà enregistrées** sur un compte devenu non-postable *après coup* restent **modifiables** (report de date, correction d'une autre ligne) — seule une ligne pointant vers un compte non-postable **qui n'était pas déjà utilisé par cette écriture** est refusée. Les **flux automatiques** (factures de vente, avoirs, factures fournisseurs, réconciliation) ne sont **pas** concernés. À la saisie, les sélecteurs de compte ne proposent plus que les comptes postables.
- **Réactiver un compte archivé** : un compte archivé peut désormais être **remis en service** depuis la page *Plan comptable* (cocher *Afficher les archivés*, puis *Réactiver*). Jusqu'ici l'archivage était irréversible depuis l'interface. La réactivation est refusée — avec un message explicite — si le compte parent est lui-même archivé. Si le **rôle** du compte a été attribué à un autre compte pendant son archivage, Kesh propose de le **réactiver sans son rôle** en un clic, plutôt que de vous laisser dans une impasse. (#269)
- **Rouvrir un exercice clôturé** : un **administrateur** peut désormais **rouvrir un exercice comptable clôturé** pour corriger une erreur découverte après coup — là où la clôture était jusqu'ici définitive. La réouverture exige un **motif obligatoire**, entièrement **tracé dans le journal d'audit** (qui, quand, pourquoi — conservation 10 ans, CO art. 957-964), et se fait depuis la page *Exercices comptables* (bouton *Réouvrir*, réservé aux administrateurs — la clôture, elle, reste accessible aux comptables). Un exercice rouvert **redevient modifiable** (saisie et correction d'écritures) puis pourra être **re-clôturé**. Garde-fou : on rouvre **du plus récent vers l'ancien** — si un exercice postérieur est encore clôturé, Kesh refuse la réouverture et indique lequel rouvrir d'abord. Le dialogue de clôture a été reformulé en conséquence (il ne parle plus d'action « définitive »).
- **Écran « Soldes de départ » — migrez votre comptabilité en une saisie** : un nouvel écran (menu *Administration → Soldes de départ*, réservé Comptable+) permet de **reprendre les soldes de votre ancienne comptabilité** : saisissez les soldes de vos comptes de bilan (actifs, passifs, capitaux propres — dont votre **report à-nouveau accumulé** sur le compte de rôle *Bénéfice reporté*) dans une grille avec **total débit/crédit et différence en direct**, puis générez en un clic **une écriture d'ouverture équilibrée datée au 1er jour de votre premier exercice**. Votre bilan est juste dès le premier jour, sans composer d'écriture OD à la main. Garde-fous : l'écran n'est actif que tant que la société n'a **aucune écriture** (sinon il se verrouille et pointe vers le journal pour corriger l'écriture d'ouverture — une OD normale, éditable) ; seuls les **comptes de bilan** sont proposés (jamais de produits/charges, qui fausseraient le résultat de l'exercice) ; le libellé de l'écriture est enregistré dans la **langue comptable** de la société.
- **Capitaux propres présentés par rôle au bilan** : le bilan affiche désormais une section **« Capitaux propres » distincte des dettes**, où vos comptes de fonds propres sont **regroupés par rôle** (capital, autres fonds propres, bénéfice/perte reporté) dans l'ordre légal (CO art. 959a) — au lieu d'être noyés parmi les passifs. Un point important pour les comptabilités **reprises d'un autre logiciel** : le **solde de report d'ouverture** que vous avez saisi sur un compte (report à-nouveau physique) apparaît sous son propre numéro de compte, tandis que le **« Résultat reporté (calculé) »** que Kesh calcule automatiquement à partir de vos exercices antérieurs reste une **ligne distincte, explicitement marquée « calculé »** — les deux ne sont jamais confondus ni additionnés par erreur. La distinction se retrouve à l'écran comme dans les exports **PDF** et **CSV** du bilan. L'équation du bilan (Actif = Dettes + Capitaux propres) reste équilibrée à tout instant.

### Modifié

- **API** : la modification d'un compte (`PUT /api/v1/accounts/{id}`) exige désormais les champs `role` et `postable` dans le corps de la requête, au même titre que `name` et `accountType`. Ce choix est délibéré : rendre ces champs facultatifs aurait permis d'**effacer silencieusement le rôle** d'un compte en corrigeant simplement son libellé. Pour retirer un rôle, envoyer explicitement `"role": null`. La lecture (`GET`) reste rétro-compatible (ajout de champs uniquement). Cela ne concerne que les intégrations qui appellent l'API directement.

### Corrigé

- **Écriture datée hors de son exercice désormais refusée** : par sécurité, une écriture comptable dont la date tombe **en dehors des bornes de son exercice** est rejetée dès la saisie (elle l'était déjà à la modification, elle l'est maintenant aussi à la création) — garantie que chaque écriture reste rattachée au bon exercice, condition de justesse du report à-nouveau.

## [0.7.0] — 2026-07-20

### Corrigé

- **« Marquer payée » ne fonctionnait pas** : cliquer sur *Marquer payée* (page Facturer / Échéancier) restait sans effet — la facture n'était pas enregistrée comme payée (erreur technique 422 côté serveur). Corrigé : le marquage manuel du paiement fonctionne de nouveau. (#249)
- **Montants TTC corrects sur la QR-facture, le PDF, l'e-mail, l'échéancier et le rapprochement bancaire** : le montant demandé par la **QR-facture**, la ligne **« Total TTC »** du PDF, la variable **`{amount}`** des e-mails, les **totaux de l'échéancier** (KPI « impayées / en retard », colonne Total, export CSV) et le **rapprochement bancaire** présentent désormais le **montant réellement dû, TVA comprise**. Auparavant, pour une entreprise assujettie à la TVA, ces montants affichaient le total **hors taxe** — un client scannant la QR-facture aurait sous-payé, et surtout **le rapprochement automatique ne proposait plus aucune correspondance** (l'encaissement TTC ne correspondait jamais au hors-taxe de la facture). Le montant hors taxe reste utilisé là où c'est correct (comptabilisation du produit). Le détail de TVA par taux sur le PDF (récapitulatif) reste à venir (#151). (#246)
- **Rappel manuel « aujourd'hui » refusé le matin** : enregistrer un rappel papier daté du jour échouait (erreur technique 422) tant qu'il était avant 14h en Suisse — le système considérait la date comme « dans le futur ». Corrigé : un rappel du jour est accepté à toute heure. (#259)
- **Accessibilité de la liste des factures** : les boutons d'action (voir / modifier / supprimer), affichés en icône seule, portent désormais un **libellé lisible par lecteur d'écran** incluant la référence de la facture, et le badge **« Impayée »** respecte le contraste minimal recommandé (WCAG AA). (#256)
- **Suppression d'une facture relancée bloquée** : supprimer une facture qui a déjà fait l'objet de rappels aurait effacé l'historique de relance (preuve de recouvrement). La suppression est désormais **refusée** avec un message clair, comme pour une facture payée ou créditée — pour retirer une telle facture, passer par un avoir. (#260)
- **Export de souveraineté complet** : l'export CSV des données incluait toutes les factures mais **omettait** la date/adresse du dernier envoi par e-mail et l'état de suspension des rappels. Ces colonnes sont désormais présentes (portabilité intégrale). (#262)

### Ajouté

- **Conditions de paiement structurées sur le contact** : la fiche contact gagne un **délai de paiement en jours** (0 à 365). À la création d'une facture pour ce contact, l'**échéance est pré-remplie automatiquement** (date de la facture + délai) et le **libellé des conditions** (« Payable à 30 jours net », « Zahlbar innert 30 Tagen »…) est généré **dans la langue du client** — les deux restent modifiables avant enregistrement. Le texte libre existant est conservé pour les contacts qui n'utilisent pas le délai structuré. Une facture ne peut plus recevoir une **échéance antérieure à sa date** (les factures existantes ne sont pas modifiées). (#245)
- **Socle des rappels débiteurs** (fondation, écrans à venir) : mise en place de la configuration des **niveaux de rappel** (délai + frais par niveau, bornés à 10'000.–) et d'une **période de grâce** par entreprise, avec un jeu de **trois niveaux par défaut** prêts à l'emploi (1er rappel, 2e rappel, dernier rappel avant poursuite) et des **modèles d'e-mail de rappel par niveau** dans les quatre langues (ton en escalade). Ces réglages seront pilotables depuis l'interface dans une prochaine étape. (#231)
- **Réglages des rappels débiteurs** (réservé à l'**administrateur**) : nouvelle page **Réglages → Rappels débiteurs** pour configurer les **niveaux de rappel** (délai en jours + frais), la **période de grâce**, avec un **échéancier prévisionnel** affiché en direct (« 1er rappel proposé 15 j après l'échéance, 2e à 25 j… ») et un rappel sur la **base contractuelle (CGV)** requise pour les frais. La page **Modèles d'e-mail** gère désormais **plusieurs types** (envoi de facture + rappel) et, pour les rappels, un **sélecteur de niveau** pour personnaliser l'objet et le corps de chaque relance. (#231)
- **Envoi des rappels débiteurs par e-mail** (backend, écran de relance à venir) : on peut désormais **envoyer un rappel par e-mail** au débiteur — un **aperçu** rendu dans la langue et le ton du niveau (1er rappel courtois → mise en demeure), avec le **PDF de la facture joint**, le montant total dû (TVA + frais de rappel cumulés) et le nombre de jours de retard calculés automatiquement. Trois modes : **aperçu**, **envoi unitaire** (avec choix du niveau) et **envoi par lot** des factures dues (jusqu'à 20 à la fois, chacune au prochain niveau, avec un compte-rendu des envois réussis et des échecs par facture). Le **destinataire est verrouillé** sur l'e-mail de la fiche contact (sécurité), l'ordre « e-mail parti ⇒ trace enregistrée » garantit qu'aucun envoi réel n'est perdu, et chaque envoi est **historisé** (preuve de recouvrement) et **journalisé**. Sans configuration SMTP, l'envoi est simplement indisponible. (#231)
- **Écran « Rappels »** (réservé aux **comptables et administrateurs**) : une nouvelle page **Rappels** liste les **factures à relancer**, groupées par débiteur, avec leur prochain niveau de rappel (ou l'état « dernier niveau atteint »). On peut **envoyer un rappel par e-mail à l'unité** (avec un aperçu modifiable dans la langue et le ton du niveau, et le choix du niveau), **en lot** (jusqu'à 20 factures, avec un compte-rendu des envois réussis et des échecs), ou **enregistrer un rappel papier** déjà envoyé hors Kesh (courrier, recommandé). Le destinataire est verrouillé sur l'e-mail de la fiche contact, et un contact sans e-mail reste visible (badge **« sans e-mail »**) : il ne peut pas recevoir d'e-mail mais le rappel papier reste possible. Une **protection anti-double-envoi** empêche qu'un double-clic n'envoie deux fois le même rappel. (#231)
- **Factures à rappels suspendus enfin visibles** : la liste des factures affiche désormais un badge **« Suspendu »** sur les factures dont les rappels sont suspendus (la **note de suspension** — « litige en cours »… — apparaît au survol), et un filtre **Rappels : Tous / Suspendus / Actifs** permet de les retrouver. Jusqu'ici, suspendre les rappels d'une facture la faisait **sortir de la liste « à rappeler » sans qu'aucun écran ne le signale** : la facture devenait introuvable et il était impossible de reprendre ses rappels. L'état de suspension est aussi disponible sur la fiche facture. Conformément à la règle « une facture suspendue ne se cache pas », elle **reste visible dans l'échéancier** quel que soit ce filtre. (#231)
- **Gestion des rappels débiteurs — moteur** (fondation, écran de relance à venir) : le système sait désormais **déterminer automatiquement les factures à relancer** (validées, impayées, échéance + délai dépassés), à **quel niveau** de rappel, en **respectant la période de grâce** et le cycle configuré — une facture arrivée au **dernier niveau reste visible** (« poursuite à envisager », jamais de disparition silencieuse). On peut **suspendre puis reprendre** les rappels d'une facture donnée (avec note) — une facture suspendue **reste dans l'échéancier et la balance âgée**, elle sort seulement de la liste « à rappeler ». On peut **enregistrer un rappel manuel** (courrier/recommandé envoyé hors Kesh, y compris en sautant directement à la mise en demeure) et **annuler** un rappel envoyé par erreur (réservé à l'administrateur). Chaque rappel est **historisé avec une copie du texte réclamé et des frais** (valeur probatoire pour un dossier de recouvrement) et intégré à l'**export souveraineté** et à la **sauvegarde**. L'écran de relance et l'envoi par e-mail arrivent dans une prochaine étape. (#231)
- **Balance âgée des créances clients** (rapport, réservé **lecture** à tous les rôles ; **export CSV** réservé aux comptables et administrateurs) : nouvel onglet **Rapports → Balance âgée** qui répartit l'**encours débiteur (TVA comprise)** par client et par **tranche d'ancienneté** — **Non échu | 1-30 | 31-60 | 61-90 | 90+ jours de retard** — arrêté à ce jour, avec un **total général** qui réconcilie avec le compte débiteurs (la colonne « Non échu » garantit que le total colle au grand livre). Chaque ligne renvoie d'un clic vers **les factures du client**. Les factures dont les rappels sont **suspendus restent comptées** (rien ne se cache). Depuis l'**échéancier** et la page **Rappels**, un lien mène directement à la balance âgée (onglet désormais **adressable** par URL). (#231)
- **Rappels visibles là où vous travaillez déjà** : la **fiche d'une facture** affiche désormais l'**historique de ses rappels** (date, niveau, canal e-mail/manuel, frais, destinataire — un rappel annulé est barré) et permet de **suspendre les rappels** (avec un motif optionnel : litige, arrangement) puis de les **reprendre**, avec le badge **« Suspendu »** au survol duquel s'affiche le motif. Le **tableau de bord** signale d'un coup d'œil « **N facture(s) à rappeler** » sur le widget *Factures ouvertes*, avec un lien direct vers l'écran Rappels (réservé aux comptables et administrateurs). Enfin, un **aller-retour de navigation** relie l'**échéancier** et l'écran **Rappels**. (#231)

### Modifié (technique)

- **Envoi de facture par e-mail désormais journalisé en cas de succès** : jusqu'ici, seuls les **échecs** d'envoi laissaient une trace dans le journal applicatif — un envoi réussi n'en laissait aucune. Un envoi réussi est maintenant journalisé (facture, destinataire, canal), comme les rappels. Utile lorsqu'un client affirme ne pas avoir reçu sa facture. (#231)
- **Version minimale requise portée à 0.7.0** : cette version introduit une évolution de base de données qui empêche de revenir en arrière vers un binaire antérieur à 0.7.0 (protection anti-downgrade — un ancien binaire ne saurait pas lire les nouveaux modèles de rappel). Sans conséquence pour une mise à jour normale. (#231)
- **Manuels admin et utilisateur enrichis** : le **manuel utilisateur** documente désormais le cycle de relances (écran Rappels, envoi unitaire/lot/manuel, recommandé pour la mise en demeure, suspension/reprise, historique) et la **balance âgée** ; le **manuel administrateur** documente les réglages de rappels (niveaux, grâce, frais/CGV, frais hors QR-facture) et la rétention légale des rappels (CO 958f, LPD). Un **test E2E de bout en bout** couvre le cycle complet des rappels. (#231)

## [0.6.0] — 2026-07-11

### Ajouté

- **Modèles d'e-mail multilingues** (réservé à l'**administrateur**) : nouvelle page **Réglages → Modèles d'e-mail** pour personnaliser l'objet et le corps de l'e-mail d'envoi de facture, dans les **quatre langues** (FR/DE/IT/EN). Des variables de substitution (`{salutation}`, `{contactName}`, `{invoiceNumber}`, `{amount}`, `{dueDate}`, `{companyName}`) sont remplacées automatiquement à l'envoi ; un modèle par défaut soigné est fourni pour chaque langue et peut être restauré à tout moment. (#224)
- **Envoi de factures par e-mail** : un bouton **« Envoyer par e-mail »** sur la fiche d'une facture validée envoie la **QR-facture PDF** directement au client — objet et message **pré-remplis dans la langue du client** (avec **civilité personnalisée** : « Chère Madame Muster », « Sehr geehrter Herr… »), modifiables avant l'envoi ; le **destinataire est verrouillé** sur l'e-mail de la fiche contact (sécurité). La fiche facture affiche ensuite « **Envoyée le … à …** », et le renvoi est possible. Se configure via les variables `KESH_SMTP_*` (indépendant de la récupération de mot de passe) — sans configuration SMTP, le bouton est simplement grisé avec explication. Les **fiches contact** gagnent une **langue de correspondance** (FR/DE/IT/EN ou héritée) et une **civilité**, et la société un **e-mail de contact** servant d'**adresse de réponse**. Attention : « envoyée » signifie remise au serveur d'envoi, pas accusé de réception. (#224)

## [0.5.2] — 2026-07-07

### Ajouté

- **Suppression d'une facture validée** (réservée à l'**administrateur**) : il est désormais possible de **supprimer définitivement** une facture validée émise par erreur ou créée pour un essai — la facture **et son écriture comptable** sont effacées ensemble, les livres restant équilibrés. Une **confirmation forte** est exigée (retaper le numéro de facture). La suppression est **refusée** si la facture est payée, si elle a déjà été créditée par un avoir, ou si elle appartient à un **exercice clos** (l'intangibilité comptable est préservée). Pour annuler une facture réellement envoyée à un client, l'**avoir** reste la voie recommandée. Toute suppression est tracée dans le **journal d'audit**. (#219)

### Modifié

- **Suppression de factures réservée à l'administrateur** : supprimer une facture (brouillon comme validée) requiert désormais le rôle **Administrateur** (la suppression d'un brouillon était auparavant accessible aux comptables). (#219)

## [0.5.1] — 2026-07-07

### Corrigé

- **Enregistrement des réglages de facturation** : sauvegarder la configuration (Paramètres → Facturation — comptes par défaut, comptes TVA, formats, journal) échouait avec une **erreur serveur 422** et aucune modification n'était enregistrée. Corrigé : le format de numérotation des avoirs, non transmis par le formulaire, est désormais optionnel et **conservé** s'il n'est pas fourni. (#216)

## [0.5.0] — 2026-07-06

Cette version met les **adresses en conformité avec le standard bancaire suisse** et enrichit le carnet de contacts.

### Ajouté

- **Adresses structurées (conformité QR-facture SIX)** : les adresses de votre société et de vos contacts se saisissent désormais en **champs séparés** — rue, numéro, NPA, localité, pays. C'est le format **structuré (type S)** exigé par les banques suisses depuis fin 2025 (l'ancienne adresse « combinée » sur une seule ligne n'est plus acceptée sur les QR-factures et les paiements pain.001). Vos **QR-factures sont désormais générées au bon format** et acceptées par les banques. Le NPA reste libre (adresses étrangères possibles).
- **Contacts : distinction personne / entreprise** : un contact de type **Personne** se saisit avec un **prénom et un nom séparés** ; un contact **Entreprise** garde sa raison sociale. Le nom d'affichage (et le nom porté sur la QR-facture) est recomposé automatiquement. Idem pour votre propre société si vous êtes en **raison individuelle** (indépendant).
- **Personnes de contact d'une entreprise** : un contact **Entreprise** peut désormais avoir une ou plusieurs **personnes de contact** (interlocuteurs : prénom, nom, fonction, email, téléphone), gérées depuis sa fiche. Ces personnes sont **purement informatives** (carnet d'adresses / relation client) — elles n'apparaissent jamais sur les factures ni les paiements.

### Note de mise à jour

La mise à jour depuis la v0.4.x est **transparente** : les nouvelles colonnes et la table des personnes de contact sont créées automatiquement au démarrage, aucune donnée n'est perdue. Pour bénéficier des QR-factures conformes, **complétez l'adresse structurée** de votre société (Paramètres) et de vos clients avant d'émettre de nouvelles factures.

## [0.4.0] — 2026-07-05

Cette version apporte la **gestion complète des factures fournisseurs et des paiements** (Epic 12) ainsi que la **comptabilité analytique par projet** (Epic 19), et modernise la base technique (Rust 1.96, dépendances à jour).

### Ajouté

- **Rapport « Rendement par projet »** (Epic 19) : nouvel onglet **Rapports → Rendement par projet** pour analyser la performance d'un projet d'investissement. Pour un projet (et ses sous-projets), il calcule le **coût investi** (charges + actifs immobilisés tagués — la trésorerie banque/caisse en est exclue pour ne pas gonfler le coût), les **revenus**, le **résultat net** (revenus − charges) et le **rendement %** (revenus / coût investi). Vue par sous-projet avec total, en mode **exercice** ou **cumulé**. Exportable en **PDF** et **CSV**. Complète le rapport de dépenses : ensemble, ils couvrent le double besoin « dépenses déductibles » et « rendement d'investissement » d'Epic 19.

- **Rapport « Dépenses par projet »** (Epic 19) : un nouvel onglet **Rapports → Dépenses par projet** liste **toutes les charges taguées sur un projet** (et ses sous-projets), groupées par sous-projet puis par compte, avec sous-totaux et total. Chaque compte se **déplie** pour montrer les écritures qui le composent (numéro, date, montant) — idéal pour vérifier et ne rien oublier au moment d'une déclaration fiscale de rénovation. Deux modes : **par exercice** ou **cumulé depuis l'origine** du projet (traverse les clôtures d'exercice). Exportable en **PDF** et **CSV**. Les montants sont hors TVA (la TVA récupérable étant portée séparément). *(Le rapport de rendement par projet suivra.)*
- **Comptabilité analytique par projet — banque & réconciliation** (Epic 19) : lorsqu'une écriture est créée **depuis un mouvement bancaire**, elle peut désormais être **affectée à un projet analytique**. Trois cas : un **rapprochement manuel** porte un projet sur toute l'écriture ; une **ventilation** (éclatement d'une transaction en plusieurs lignes) permet d'affecter **un projet par ligne** (par exemple répartir un versement entre deux chantiers) ; et une **règle d'affectation automatique** peut porter un **« projet par défaut »** — chaque transaction réconciliée via cette règle est alors automatiquement taguée, sans ressaisie. Si le projet par défaut d'une règle a été archivé entre-temps, la réconciliation de cette transaction est signalée en échec (invitant à corriger la règle) plutôt que de taguer un projet clos. Complète la couverture analytique sur **tous** les flux de saisie (factures, écritures manuelles, banque).
- **Comptabilité analytique par projet — factures de vente** (Epic 19) : une facture client peut désormais être **affectée à un projet analytique** dès sa création (ou en éditant le brouillon). À la validation de la facture, le projet est propagé sur toutes les lignes de l'écriture de vente — les **revenus du projet** alimentent ainsi le futur rapport de rendement (revenus / coût investi), en face des dépenses déjà capturées via les factures fournisseurs. Si la facture est annulée par un avoir, la contre-passation reprend le même projet (le solde analytique revient à zéro). Un projet archivé entre le brouillon et la validation est refusé à la validation (aucune nouvelle opération sur un projet clos).
- **Comptabilité analytique par projet — écritures manuelles** (Epic 19) : dans le formulaire de **saisie d'écriture** (création comme modification), chaque ligne peut désormais être **affectée à un projet analytique** via un sélecteur dédié (projets et sous-projets). Contrairement aux factures (affectées globalement), le tag se fait **ligne par ligne** — une même écriture de régularisation peut ainsi ventiler ses charges sur plusieurs projets. Le projet de chaque ligne est visible sur le **détail de l'écriture**. La colonne n'apparaît que si des projets actifs existent ; les écritures historiques taguées sur un projet depuis archivé restent modifiables (le tag est conservé).
- **Comptabilité analytique par projet — factures fournisseurs** (Epic 19) : à l'enregistrement d'une facture fournisseur, vous pouvez désormais l'**affecter à un projet analytique** (créé au préalable dans *Administration → Projets analytiques*). Le projet est propagé sur l'écriture d'achat comptabilisée (et sur son règlement / son annulation), ce qui permettra d'analyser **toutes les dépenses d'un projet** (ex. une rénovation déductible) et son rendement. Idéal pour ne rien oublier au moment de la déclaration fiscale. *(Le tagging des écritures manuelles, factures de vente et transactions bancaires suivra.)*
- **Factures fournisseurs & règlement** (#191) : Kesh modélise désormais les **factures reçues de vos fournisseurs**. Enregistrer une facture fournisseur (fournisseur, date, échéance, lignes avec compte de charge et taux de TVA) poste automatiquement l'**écriture d'achat** (charge + impôt préalable récupérable + dette envers le créancier) et la met au statut « ouverte ». Vous pouvez ensuite la **payer en un clic** via un choix simple : soit un **virement bancaire** (en choisissant le compte bancaire à débiter — préparant l'export pain.001 à venir), soit un **compte interne** librement choisi dans votre plan comptable (caisse, carte de crédit, Twint…). Le paiement génère l'écriture de règlement et solde la dette du fournisseur. Une facture ouverte peut aussi être **annulée** (contre-passation automatique de l'écriture d'achat). Les coordonnées de paiement (IBAN/QR-IBAN, référence) peuvent être saisies sur la facture, indépendamment du mode de règlement choisi. Accessible depuis l'entrée **« Factures fournisseurs »** du menu (rôle Comptable ou Administrateur).
- **Paiement par fichier pain.001** (#191) : pour régler vos fournisseurs par **virement** sans ressaisir les coordonnées dans l'e-banking, Kesh génère désormais un **fichier de paiement ISO 20022 `pain.001.001.09`** (Swiss Payment Standards / SIX). Le flux se fait en **deux temps** : (1) vous sélectionnez les factures fournisseurs ouvertes à payer par virement et le compte bancaire à débiter → Kesh crée un **lot** et produit le fichier XML téléchargeable (rien n'est encore comptabilisé) ; vous importez ce fichier dans l'e-banking de votre banque ; (2) une fois le virement exécuté, vous **confirmez le lot** dans Kesh → les écritures de règlement sont comptabilisées et les factures passent « payées ». Un lot peut être **annulé** avant confirmation. Les factures avec QR-IBAN utilisent la référence QRR, les autres une référence libre. Accessible depuis l'entrée **« Paiements fournisseurs »** du menu (rôle Comptable ou Administrateur).
- **Import de factures fournisseurs depuis un dossier** (#194) : déposez vos factures reçues (PDF ou images **porteurs d'un Swiss QR-facture**) dans un **dossier surveillé** sur le serveur/NAS, puis cliquez **« Importer le dossier »** — Kesh lit le dossier, **décode le QR côté serveur** (créancier, IBAN/QR-IBAN, montant, référence), **archive une copie du fichier** comme justificatif récupérable, et crée pour chaque facture une entrée **« à compléter »**. Vous n'avez plus qu'à choisir le fournisseur et saisir les lignes (compte de charge, TVA) : les coordonnées de paiement sont déjà remplies, et Kesh vérifie que le total des lignes correspond au montant du QR. La facture complétée entre dans la comptabilité comme une facture fournisseur normale (payable ensuite par virement pain.001 ou compte interne), avec un lien **« Voir la facture d'origine »** sur son détail. Un rapport d'import liste les factures créées et les fichiers rejetés (type non supporté, QR absent, doublon…). Accessible depuis l'entrée **« Importer des factures »** du menu (rôle Comptable ou Administrateur). *(Configuration des dossiers : voir le manuel administrateur.)*
- **Scan du QR-facture à la saisie** (#191) : lors de l'enregistrement **manuel** d'une facture fournisseur, un bouton **« Scanner un QR-facture »** permet de charger une **image** du QR-facture (photo ou capture d'écran) — le QR est **décodé dans le navigateur** (sans caméra, compatible accès HTTP LAN) et Kesh **pré-remplit** automatiquement l'IBAN/QR-IBAN, la référence et le montant attendu, et affiche le nom du créancier détecté. Vous complétez ensuite les lignes comptables (compte de charge, TVA) comme d'habitude. Conclut la fonctionnalité **Factures fournisseurs & paiements** (#191).

### Modifié

- **Base technique modernisée** : passage à **Rust 1.96** (édition 2024) et rafraîchissement complet des dépendances backend. Sans impact fonctionnel — inclut les dernières corrections de sécurité des bibliothèques réseau/chiffrement (TLS, HTTP, runtime asynchrone). La mise à jour depuis une version précédente est transparente (aucune migration de données requise ; les nouvelles tables et colonnes analytiques sont créées automatiquement au démarrage).

---

## [0.3.2] — 2026-06-27

### Ajouté

- **Avoirs (notes de crédit)** (#186, #184) : une facture validée peut désormais être **annulée** en créant un **avoir** qui lui est lié. L'avoir reprend toutes les lignes de la facture, reçoit un numéro de sa propre séquence (`AV-2026-0001`), et génère automatiquement l'**écriture de contre-passation** (TVA comprise, une ligne par taux) : le solde du client revient à zéro et la facture passe au statut « annulée ». Le tout en une seule étape, depuis le bouton **« Créer un avoir »** d'une facture validée (rôle Comptable ou Administrateur). Un PDF « Avoir » est téléchargeable (sans QR-facture, avec référence à la facture d'origine). Le décompte TVA de la période exclut correctement la facture annulée. La création d'avoir est refusée sur une facture déjà encaissée (le remboursement d'une facture payée viendra plus tard). Cette fonctionnalité remplace la contre-passation manuelle décrite jusqu'ici dans le manuel.

---

## [0.3.1] — 2026-06-27

### Corrigé

- **Message clair lorsqu'une écriture comptable ne peut pas être supprimée** (#184) : tenter de supprimer une écriture qui a été générée par une facture validée affichait un message générique et déroutant (« Référence invalide »). Kesh explique désormais la situation et la marche à suivre : *« Cette écriture comptable a été générée par une facture validée et ne peut pas être supprimée directement. Annulez d'abord la facture concernée. »* (traduit dans les 4 langues). L'intégrité des données était déjà garantie auparavant — seul le message gagne en clarté.

### Sécurité

- **Moins de données sensibles dans les journaux en mode debug** (#185) : en mode de journalisation détaillé (`RUST_LOG=debug`), la couche base de données pouvait écrire le texte complet de chaque requête SQL **avec ses valeurs** dans les logs (données métier en clair sur disque). Kesh limite désormais par défaut la journalisation de la couche SQL au niveau `warn` même en mode debug applicatif ; l'administrateur qui a réellement besoin du détail SQL doit l'activer explicitement (`RUST_LOG=debug,sqlx=debug`). Le fichier `.env.example` documente cet avertissement. Aucune fuite réseau n'était en cause (journaux locaux) ; cette amélioration réduit la quantité de données métier persistées en clair.

---

## [0.3.0] — 2026-06-26

### Ajouté

- **Comptabilisation de la TVA & achats avec impôt préalable** (#180) : Kesh comptabilise désormais réellement la TVA dans le plan comptable, au-delà du simple calcul du rapport. Concrètement :
  - **Comptes TVA dans le plan comptable** : trois comptes TVA standard du plan suisse sont ajoutés et configurables depuis **Paramètres → Facturation** — *TVA due* (`2200`), *Impôt préalable* (`1171`, TVA récupérable sur les achats) et *Décompte TVA* (`2206`). Les installations existantes sont complétées automatiquement sans toucher aux comptes déjà utilisés.
  - **TVA due comptabilisée à la validation des factures de vente** : valider une facture génère désormais une écriture comptable complète — créance TTC au débit, produit HT au crédit et **une ligne de TVA due par taux** — au lieu des seules lignes hors taxe. Le taux figé sur chaque ligne de facture est utilisé (une modification ultérieure des taux n'altère pas les factures déjà validées).
  - **Saisie assistée des achats avec impôt préalable** : un assistant pré-remplit l'écriture d'un achat avec TVA récupérable (charge / impôt préalable / fournisseur TTC) à partir d'un taux, depuis le journal des écritures — sans nouvelle entité « facture d'achat ».
  - **Décompte TVA complet** : le rapport TVA affiche désormais la **TVA récupérable réelle** (solde du compte d'impôt préalable au grand livre) et le **solde net dû à l'AFC** (TVA due − récupérable), y compris pour une période d'achats sans vente.
  - **Réconciliation rapport ↔ grand livre** : le décompte est recoupé avec les écritures comptables. Si une écriture validée a été modifiée à la main et ne correspond plus à la TVA facturée, un **bandeau d'alerte** signale l'écart (non bloquant) pour inviter à vérifier — garantissant que « les montants du décompte correspondent aux écritures ». Le décompte et son écart figurent aussi dans les exports PDF et CSV.

  > Limitation : le **format de décompte officiel AFC / e-décompte ESTV** et la méthode des taux de la dette fiscale nette (TDFN) restent hors périmètre pour l'instant.

---

## [0.2.0] — 2026-06-12

### Added

- **API externe à clé d'accès (PAT)** (#100) : Kesh peut désormais être branché à des intégrations externes — IA (Claude API, ChatGPT, agents), scripts, ETL, dashboards BI ou ERP — via des **clés d'accès API** liées à une entreprise. Chaque clé a une portée *lecture seule* ou *lecture-écriture* et agit au nom de l'utilisateur qui l'a créée, sans partager d'identifiants. Les clés se créent et se révoquent depuis la nouvelle page **Paramètres → Clés API** (`/settings/api-keys`) ; le secret n'est affiché qu'une seule fois et seul son condensé est conservé côté serveur. L'authentification se fait par l'en-tête HTTP `Authorization: Bearer kesh_pat_…` sur les routes `/api/v1/*` existantes. Guide complet d'intégration (exemples curl, Python, JavaScript, MCP) : [`docs/api-external.md`](docs/api-external.md).
- **Récupération de mot de passe par email** (#122) : un utilisateur qui a oublié son mot de passe peut désormais le réinitialiser **en self-service** via un lien envoyé par email — sans intervention d'un administrateur ni accès SSH au serveur. Le lien « Mot de passe oublié ? » apparaît sur l'écran de connexion lorsque la fonctionnalité est activée (`KESH_FEATURE_FORGOT_PASSWORD=true` + configuration SMTP, cf. `.env.example`). Le lien de réinitialisation est valable **30 minutes**, à **usage unique**, et la réinitialisation déconnecte toutes les sessions actives du compte. Conçu contre l'énumération de comptes (réponse générique systématique) et avec limitation de débit (5 demandes / 15 min / IP). Le recovery *break-glass* administrateur (`KESH_ADMIN_*` via `.env`) reste disponible en dernier recours (SMTP en panne, compte sans email). Un champ **email** (optionnel) est désormais proposé à la création du 1er administrateur et dans la gestion des utilisateurs. Voir le **manuel administrateur** (§ Récupération de mot de passe par email) et le **manuel utilisateur** (§ Récupération de mot de passe).
- **Export / import complet d'une installation** (#112) : un administrateur peut désormais **exporter toute l'installation Kesh** (toutes les sociétés, les utilisateurs et les données système) dans un fichier `.keshbackup` unique, puis le **réimporter sur une autre instance** — pour **migrer** ou **restaurer** une installation **sans accès SSH ni ligne de commande**. L'export se déclenche depuis **Administration → Sauvegarde complète** (`/admin/backup`), l'import depuis **Administration → Restaurer / Importer** (`/admin/restore`). L'import est une opération **destructrice** qui **remplace l'intégralité des données** de l'instance et **déconnecte** l'utilisateur (reconnexion avec les identifiants de l'instance importée) ; une **sauvegarde automatique de l'état courant est créée côté serveur avant l'import** (filet de sécurité). Des garde-fous refusent un fichier corrompu, un schéma incompatible ou une sauvegarde exigeant une version de Kesh plus récente que l'installation (protection anti-downgrade). Fonction **réservée au rôle Admin** (inaccessible via clé API). ⚠️ Le fichier `.keshbackup` contient des données sensibles (hash de mots de passe, condensés de clés API, jetons de session) : à conserver et transmettre comme un **secret**. À distinguer de l'**export global per-société** (CSV/ZIP, `/export`) destiné à l'extraction comptable d'une seule entreprise. Voir le **manuel d'administration** (§ Sauvegarde et restauration → Export/import via l'interface Kesh) pour la matrice des méthodes (Hyper Backup DSM / `mariadb-dump` / export-import UI).

### Sécurité

- **Création du 1er administrateur désormais atomique** (#133) : l'endpoint de configuration initiale (`POST /setup/admin`) fermait une fenêtre de *race condition* (TOCTOU) qui pouvait, sous deux requêtes concurrentes avec des identifiants distincts, créer **deux comptes administrateur** non concertés au lieu d'un seul. La vérification « aucun utilisateur n'existe » et la création du premier admin s'exécutent maintenant dans une **transaction unique sérialisée par un verrou exclusif**, garantissant qu'au plus un administrateur est créé même en cas d'accès simultané pendant la fenêtre d'onboarding. Comportement utilisateur inchangé en usage nominal.

---

## [0.1.8] — 2026-06-04

Correctif issu du dogfooding live sur prod NAS Synology v0.1.7.

### Fixed

- **Numéro de version affiché incorrect** (#159) : le pied de page (ainsi que les écrans de connexion et de configuration) affichaient « Kesh v0.1.0 » au lieu de la version réellement installée. La version affichée provient désormais du **backend au runtime** (champ `version` de la réponse `/health`, résolu depuis `crates/kesh-api/Cargo.toml` à la compilation), garantissant qu'elle correspond toujours au binaire qui tourne — sans dépendre d'un fichier frontend à mettre à jour manuellement à chaque release. Cause : pieds de page codant la version en dur + source frontend (`package.json`) jamais bumpée.

---

## [0.1.7] — 2026-06-03

Correctifs issus du dogfooding live de la facturation sur prod NAS Synology v0.1.6.

### Changed

- **Compte bancaire — aide sur le champ QR-IBAN** (#155) : un texte explicatif précise désormais que le champ « QR-IBAN » ne doit être rempli **que** si la banque a fourni un QR-IBAN dédié aux QR-factures (identifiant 30000–31999), et qu'il faut sinon le laisser vide (l'IBAN normal suffit pour générer des QR-factures). En cas de saisie d'un IBAN qui n'est pas un QR-IBAN, le message d'erreur est désormais actionnable (« laissez ce champ vide ») au lieu du message technique « QR-IID … hors plage 30000-31999 ».

### Fixed

- **Suite de tests `journal_entries` auto-réparante** (#140) : 20 tests backend échouaient en `FiscalYearClosed` lorsque l'exercice de l'année courante était clos en base de développement (clôture manuelle pendant le dogfooding, ou test antérieur le laissant clos). Le helper de test garantit désormais un exercice **ouvert** couvrant la date du jour, indépendamment de l'état du seed. Aucun impact sur l'application ni sur les données — correctif purement test (dette technique catégorie A levée avant le passage à v0.2).

---

## [0.1.6] — 2026-06-03

Correctifs et améliorations issus du dogfooding live de la facturation sur prod NAS Synology v0.1.5.

### Fixed

- **« Voir l'écriture comptable » → page 404** (#148) : sur une facture validée, le bouton menait à une route inexistante. Ajout de la **page détail d'une écriture comptable** (`/journal-entries/{id}`, affichage des lignes débit/crédit + comptes + totaux) et de l'endpoint backend `GET /api/v1/journal-entries/{id}` (scopé société, 404 anti-énumération cross-tenant).

### Changed

- **Facture — placement des boutons d'ajout de ligne** (#149) : « Ligne libre » et « Depuis catalogue » sont désormais placés **sous le tableau des lignes** (au lieu de l'en-tête) pour un flux d'ajout plus naturel.
- **Facture — bouton d'impression** (#150) : « Télécharger PDF » renommé **« Imprimer / Télécharger PDF »** avec une icône imprimante, pour la découvrabilité (la fonction existait déjà mais n'était pas identifiée comme l'impression).

---

## [0.1.5] — 2026-06-03

Correctifs issus du dogfooding live sur prod NAS Synology v0.1.4 (déploiement HTTP réseau local).

### Fixed

- **Pages « Facturer » et « Échéancier » blanches en déploiement HTTP** (#145) — sur une installation servie en HTTP sur le réseau local (sans HTTPS), ces deux pages s'affichaient entièrement vides. Cause : une fonctionnalité du navigateur (`crypto.randomUUID`) n'est disponible qu'en contexte sécurisé (HTTPS ou `localhost`) et provoquait une erreur bloquant le rendu. Les pages se chargent désormais correctement quel que soit le mode de déploiement.
- **Liste déroulante « Compte parent » non défilable** (#143) — lors de la création d'un compte dans le plan comptable, la liste des comptes parents n'était pas défilable quand elle dépassait la hauteur de l'écran (cas d'un plan comptable suisse complet), rendant les comptes du bas inaccessibles. La liste est désormais plafonnée en hauteur et défilable. Le correctif s'applique à toutes les listes déroulantes longues de l'application.

---

## [0.1.4] — 2026-06-01

Hotfix UX consolidé suite à dogfooding live sur prod NAS Synology v0.1.3 : CRUD complet des comptes bancaires post-onboarding (le seul endpoint existant `POST /api/v1/onboarding/bank-account` refusait les appels post-onboarding), restructuration de la sidebar avec groupes collapsibles, ajout des 4 pages orphelines précédemment accessibles uniquement via URL directe, widget homepage avec soldes calculés.

### Added

- **CRUD `bank_accounts` post-onboarding** (`POST` / `PUT` / `DELETE` `/api/v1/bank-accounts`) — création, édition complète et soft-delete (archivage) accessibles depuis Administration → Comptes bancaires (Comptable+). Transition primary silencieuse atomique POST/PUT (l'ancien primary est démoté automatiquement avec audit `details_json.trigger = "primary_transition"`). Audit log à 3 actions (`bank_account.created`, `bank_account.updated`, `bank_account.archived`) cohérent CO Art. 958f.
- **Soft-delete via `archived` BOOLEAN** sur `bank_accounts` (migration `20260531000001_bank_accounts_archived.sql`, non-breaking) — préserve audit + historique de transactions. Toggle « Afficher les archivés » côté UI. Refus 412 sur archivage si transactions existent (`BANK_ACCOUNT_HAS_TRANSACTIONS`) ou si compte principal avec d'autres comptes actifs (`BANK_ACCOUNT_CANNOT_ARCHIVE_PRIMARY`). Archivage du primary unique autorisé.
- **Solde calculé serveur-side** sur `GET /api/v1/bank-accounts` (champ `currentBalance: Decimal | null`) — agrégation `SUM(debit) - SUM(credit)` sur `journal_entry_lines` du `journal_account_id` lié. Affiché sur la page d'accueil (par compte + total liquidités) et sur la page Comptes bancaires. `null` si le compte n'a pas de `journal_account_id` configuré (lien plan comptable manquant).
- **Sidebar collapsible** via `<details>`/`<summary>` HTML natif (a11y intégrée) avec persistence de l'état via `localStorage` (SSR-safe). 3 groupes structurés : Quotidien (déplié par défaut), Mensuel (déplié), Administration (replié). Auto-expand du groupe contenant la route active (UX + screen reader).
- **5 pages orphelines ajoutées à la sidebar** (Administration) : Plan comptable, Exercices comptables, Comptes bancaires, Profils bancaires, Règles d'affectation. Précédemment accessibles uniquement via URL directe.
- **Guide de démarrage utilisateur** (`docs/user-guide/fr/getting-started.md`) avec section dédiée à la liaison `bank_account` ↔ plan comptable et au cas multi-comptes (sous-comptes auxiliaires 1030.001/1030.002).

### Changed

- **Widget « Comptes bancaires » homepage** : affiche les soldes calculés au lieu d'un CTA configuration. Retiré complètement du DOM si aucun compte bancaire n'existe (`{#if bankAccounts.length > 0}`). Total liquidités affiché en pied de carte si plusieurs comptes.
- **Sidebar restructurée** : entrée « Payer » (qui pointait vers `/bank-accounts` — nom trompeur car c'était la configuration, pas un flow paiement) renommée en « Comptes bancaires » et déplacée sous Administration. Items « Export global » / « Paramètres » / admin-only (« Utilisateurs », « Facturation ») fusionnés dans Administration plutôt que dispersés en groupes séparés.
- **Page `/settings` — section Comptes bancaires** : remplacement du bouton « Modifier » (qui affichait un toast `notYet()`) par un lien direct vers `/bank-accounts`. Texte d'aide explicite.

### Fixed

- **Cohérence cross-fichier du flag `archived`** (FINDING-1/2/6 Pass 3 Opus spec-validate) : la fonction repo `bank_accounts::find_primary` (utilisée par `routes/invoice_pdf.rs:83` pour le QR Bill) filtre désormais `archived = FALSE` — sans ce filtre, un primary archivé continuerait à servir d'IBAN pour les PDF de factures alors qu'il n'apparaît plus côté UI (état fantôme). Idem pour `set_journal_account_id_for_company`, `update_for_company`, `archive_for_company` — un PATCH/PUT/DELETE sur un compte archivé retourne désormais 404 anti-énumération (KF-002). 7 call sites cross-modules patchés (`bank_imports.rs:862, 1006` + `reconciliation.rs:349, 629, 1962, 2278, 2699`) — empêche la création de nouvelles `bank_transactions` ou réconciliations manuelles sur un compte archivé.
- **PATCH `/bank-accounts/{id}` (legacy 8-5a-zero) — cohérence audit log** : l'event `bank_account.updated` émis par le PATCH inclut désormais `details_json.trigger = "journal_account_link"` (cohérent avec le PUT qui émet `trigger = "full_update"`). Sans ce champ, un script audit qui filtre par trigger raterait toutes les écritures PATCH.

### Removed

- **Fonction `notYet()` dans `/settings`** (plus utilisée après remplacement du bouton « Modifier » par lien direct).

---

## [0.1.3] — 2026-05-31

Hotfix critique : déblocage des déploiements LAN strict HTTP-only (cookies session inutilisables sans HTTPS dans v0.1.2).

### Corrections

- **Cookies session sur HTTP-only LAN** (Issue #136) : avant v0.1.3, les cookies `kesh_access_token` et `kesh_refresh_token` portaient systématiquement le flag `Secure` en mode production (sauf en mode test E2E). Le browser refuse de stocker/envoyer un cookie `Secure` sur une connexion HTTP non-TLS, ce qui rendait Kesh inutilisable sur tout déploiement LAN privé sans HTTPS (domaine RFC 8375 `*.home.arpa`, NAS Synology derrière Traefik HTTP sans Let's Encrypt, etc.) : l'utilisateur pouvait se logger côté backend mais aucun cookie ne persistait, déclenchant une boucle d'erreurs 401 sur tous les calls subséquents. Le couplage à `KESH_TEST_MODE` (qui active aussi les endpoints dangereux `/api/v1/_test/*`) interdisait par ailleurs tout workaround propre. Désormais découplé via une variable d'environnement dédiée `KESH_COOKIE_SECURE` (défaut `true` — sécurité préservée). Les déploiements LAN HTTP-only peuvent passer à `false` avec warning explicite au boot ; les autres déploiements continuent en mode sécurisé sans changement.

### Ajouts

- **Variable d'environnement `KESH_COOKIE_SECURE`** : contrôle explicitement le flag `Secure` des cookies session, indépendamment du mode test E2E. Valeurs `true`/`1` (défaut) ou `false`/`0` ; toute autre valeur (`True`, `yes`, `on`, espaces, etc.) refuse le démarrage pour éviter toute ambiguïté. Documentée dans `.env.example` avec warning de sécurité explicite et alternative HTTPS recommandée. Le manuel administrateur ajoute une sous-section dédiée au déploiement LAN HTTP-only avec procédure et matrice de risque.

---

## [0.1.2] — 2026-05-31

Évolutions de l'expérience d'installation pour s'aligner sur les standards des applications self-hosted modernes (Jellyfin, Bitwarden, Vaultwarden) et URL HTTP standard.

### ⚠️ Action requise — upgrade v0.1.1 → v0.1.2

**Si vous avez changé votre mot de passe administrateur via l'UI depuis l'installation v0.1.0/v0.1.1**, vous devez **retirer `KESH_ADMIN_PASSWORD` de votre `.env`** AVANT de redémarrer en v0.1.2. Sinon, le password sera resetté au password de `.env` au prochain boot (mécanisme **Recovery break-glass** déclenché automatiquement quand un admin existe avec le même `KESH_ADMIN_USERNAME` mais un hash différent). Visible dans les logs Docker au démarrage : `🔓 Recovery effectué — RETIRER LES VARS DE .ENV`.

**Aucune action requise** si vous n'avez pas changé votre mot de passe administrateur depuis l'installation : le boot v0.1.2 détecte que le hash en base correspond toujours à `KESH_ADMIN_PASSWORD`, ne touche pas au password, et émet un simple warning (« retirer les vars de .env ») qui disparaîtra dès que vous les retirez.

### Ajouts

- **Onboarding self-service au 1er démarrage** : sur une nouvelle installation avec une base de données vide, Kesh affiche désormais un écran **« Bienvenue dans Kesh »** au lieu d'exiger une édition `.env` préalable. L'administrateur initial est créé via un formulaire web (`/setup`) — pattern conforme à Jellyfin/Bitwarden/Sonarr/Vaultwarden. L'écran de setup est automatiquement désactivé (`410 Gone`) dès qu'un administrateur existe. Variables `KESH_ADMIN_USERNAME` / `KESH_ADMIN_PASSWORD` deviennent **optionnelles** et conservent un double-usage : (a) **bootstrap déclaratif** (CI, Test, déploiements automatisés) si renseignées sur DB vide, (b) **recovery break-glass** si un administrateur existe avec ce username mais un mot de passe différent (cf. ci-dessous). **⚠️ Sécurité** : avant le 1er démarrage en production, bloquez l'accès réseau public — la personne qui touche `/setup` en premier devient administrateur. Recommandé : binder loopback `127.0.0.1` ou LAN privé en attendant la création du compte. (closes #121)

- **Recovery break-glass administrateur** : si vous perdez votre mot de passe administrateur, vous pouvez désormais le réinitialiser en renseignant `KESH_ADMIN_USERNAME` et `KESH_ADMIN_PASSWORD` dans `.env` puis en redémarrant le container. Le hash de l'administrateur correspondant est resetté en transaction atomique (avec rollback automatique si l'audit log fail), ses sessions actives (refresh tokens) sont révoquées, et un événement `admin_break_glass_reset` est enregistré dans le journal d'audit (conservation 10 ans Swiss CO Art. 958f). Un warning préventif est émis dans les logs avant l'UPDATE : « ⚠️ Recovery break-glass déclenché — si vous avez changé votre mdp via l'UI, votre mdp sera écrasé par KESH_ADMIN_PASSWORD ». Procédure complète step-by-step dans le manuel administrateur (section « J'ai oublié mon mot de passe administrateur »). **Pensez à retirer les variables `KESH_ADMIN_*` de votre `.env` après recovery** — un warning persistant le rappelle dans les logs à chaque boot tant qu'elles traînent. (closes #121)

### Modifié

- **Port d'écoute par défaut : 3000 → 80** : Kesh écoute désormais sur le port HTTP standard (80) au lieu de 3000. L'URL d'accès est `http://kesh.local` (sans suffixe `:port`), conforme à ce que les utilisateurs attendent d'une application web. Le container Docker tourne en root et peut donc bind ce port privilégié sans configuration supplémentaire ; le bind loopback `127.0.0.1` de la prod est conservé.

  **⚠️ Breaking de configuration — utilisateurs existants v0.1.1, choisissez l'une des deux procédures :**

  1. **Adopter le nouveau défaut 80** (recommandé) : retirer la ligne `KESH_PORT=` de votre `.env` si présente. Le mapping `docker-compose.{prod,dev}.yml` (`127.0.0.1:80:80` ou `80:80`) prend effet automatiquement au prochain `docker compose up`. URL d'accès : `http://localhost`.

  2. **Garder le port 3000** (si conflit port 80, ex. Synology DSM Web Station, ou mode `cargo run` natif sur Linux non-root) : conserver `KESH_PORT=3000` dans `.env` ET **éditer directement** `docker-compose.prod.yml` (ou `docker-compose.dev.yml` selon votre déploiement) pour remplacer la ligne du mapping :
     ```yaml
     # Avant :
     # - "127.0.0.1:80:80"
     # Après :
     - "127.0.0.1:3000:3000"
     ```
     ⚠️ Ne PAS utiliser `docker-compose.override.yml` pour surcharger : Docker Compose **concatène** les listes `ports:` (ne les remplace pas) — le mapping `80:80` resterait actif et échouerait sur un hôte avec port 80 occupé. L'édition directe est la procédure officielle (alignée avec le manuel administrateur, section « Changer le port d'écoute »). URL d'accès : `http://localhost:3000`.

  Le manuel administrateur (section « Changer le port d'écoute (conflit port 80, ex. Synology DSM) ») détaille les 4 options d'override (remap host, override KESH_PORT, IP dédiée macvlan, mode dev natif).

---

## [0.1.1] — 2026-05-29

Hotfix post-déploiement v0.1.0 : corrections et améliorations opérationnelles découvertes lors du premier déploiement en production sur NAS Synology. Cette release embarque les **2 stories critiques** de l'épic hotfix (logs fichier + déblocage du premier démarrage). Les stories restantes de l'épic (break-glass admin reset, port 80 par défaut) sont reportées à une release ultérieure.

### Ajouts

- **Logs fichier avec rotation** : en plus de la sortie standard (`docker logs`), Kesh peut désormais écrire ses logs dans un fichier avec rotation automatique (quotidienne, horaire, ou désactivée), conservés sur le disque et inclus dans le backup. Activé par défaut en production (répertoire `./log/`). Configurable via `KESH_LOG_FILE_PATH`, `KESH_LOG_FILE_ROTATION`, `KESH_LOG_FILE_MAX_FILES` et `KESH_LOG_FILE_FORMAT` (format lisible ou JSON structuré). (#119)

### Corrections

- **Premier démarrage débloqué (catch-22 onboarding)** : sur une nouvelle installation avec une base de données vide, il était impossible de terminer la configuration — l'utilisateur administrateur du `.env` n'était jamais créé tant qu'aucune entreprise n'existait, alors que la création d'une entreprise passe par un assistant qui exige justement d'être connecté. Désormais, au tout premier démarrage, Kesh crée automatiquement un compte administrateur **et** une entreprise provisoire, ce qui permet de se connecter immédiatement et de compléter l'onboarding. Une bannière non-bloquante rappelle, pendant l'assistant, que l'entreprise porte un nom provisoire jusqu'à la saisie des vraies coordonnées. (#120)

---

## [0.1.0] — 2026-05-27

Première version publique de Kesh. Cette release fournit un système comptable complet et conforme au Code des obligations suisse (Art. 957a et suivants) ainsi qu'à l'Ordonnance OLICo (RS 221.431), prêt pour un usage productif chez un indépendant, une PME ou un fiduciaire mono-poste.

### Ajouts

#### Comptabilité

- **Plan comptable suisse PME** intégré (Sterchi adapté) avec création et modification de comptes (numérotation libre, type Actif/Passif/Charge/Produit, comptes inactifs archivables).
- **Saisie d'écritures comptables** en partie double avec validation automatique de l'équilibre Débit = Crédit avant enregistrement.
- **Journaux comptables** : Banque, Caisse, Achats, Ventes, Opérations Diverses (OD) ; sélection automatique du journal selon le contexte.
- **Audit-trail complet** (table `audit_log`) : chaque écriture comptable créée, modifiée ou supprimée est tracée avec l'utilisateur, la date, et un snapshot direct des données — conforme à l'exigence OLICo Art. 9 (intégrité des supports modifiables).
- **Exercices comptables** : création et clôture d'exercices, blocage automatique des écritures sur exercice clôturé (impossibilité de modifier les écritures d'un exercice fermé).
- **Recherche et filtrage** des écritures par date, montant, journal, description (recherche full-text MariaDB).
- **Tooltips pédagogiques** sur les concepts comptables (Débit, Crédit, etc.) pour faciliter la prise en main par les utilisateurs non-comptables.

#### Facturation et QR Bill

- **Création de factures** avec lignes libres ou références au catalogue de produits.
- **QR Bill 2.2** conformes au standard SIX Interbank Clearing (norme suisse en vigueur depuis 2020), avec génération du Swiss QR Code intégré au PDF de la facture.
- **Téléchargement PDF** des factures validées, archivage local et envoi par email possible.
- **Échéancier des factures** : suivi des échéances, marquage manuel des paiements reçus, export CSV pour rapprochement bancaire.
- **Workflow facture** : brouillon → validée → payée (avec transition par marquage manuel ou réconciliation bancaire automatique).

#### Import bancaire et réconciliation

- **Import CAMT.053** (ISO 20022) — standard universel des relevés bancaires en Suisse et en Europe — avec parsing des transactions, références, soldes d'ouverture et clôture.
- **Import CSV multi-encodage** avec création de profils banque réutilisables (mapping colonnes, séparateurs, encoding UTF-8/Latin-1/Windows-1252, format de date, séparateur décimal). Support des particularités de chaque banque suisse (PostFinance, UBS, Raiffeisen, Banques Cantonales, etc.).
- **Détection automatique des doublons** : si le même fichier est importé deux fois, ou si les mêmes transactions apparaissent dans deux relevés qui se chevauchent, Kesh prévient et propose de fusionner ou ignorer.
- **Commit partiel** : si quelques lignes d'un fichier sont invalides (formats incorrects), les autres lignes correctes sont importées et un rapport détaillé liste les rejets ligne par ligne.
- **Avertissement de balance** : si le solde de clôture déclaré dans le fichier ne correspond pas au solde calculé, Kesh prévient et demande confirmation avant import.
- **Réconciliation manuelle** : sélection d'une transaction bancaire et d'une facture (ou groupe d'écritures) pour les marquer comme rapprochées.
- **Réconciliation automatique par règles** : définition de règles d'affectation (par libellé, montant, contrepartie) qui rapprochent automatiquement les nouvelles transactions importées. Suggestions multi-candidats si plusieurs règles matchent.
- **Split de transaction** : une transaction bancaire peut être éclatée en plusieurs écritures comptables (utile pour les versements groupés).

#### Rapports comptables

- **Bilan** : actif/passif à une date donnée avec sous-totaux par classe de compte.
- **Compte de résultat** : produits et charges sur une période avec calcul du résultat net.
- **Balance** : tous les comptes avec leurs soldes débiteurs/créditeurs, filtres par classe.
- **Journal** : chronologie complète des écritures sur une période.
- **Exports** : tous les rapports exportables en PDF (mise en page officielle) et CSV (analyse Excel/LibreOffice).

#### Export global de souveraineté

- **Export ZIP global** d'une company (Story 9-2b) : un seul ZIP contient tous les rapports comptables, toutes les factures PDF, tous les imports bancaires, le journal complet, avec un hash SHA-256 d'intégrité. Permet à tout moment de retrouver vos données dans un format consultable et vérifiable, indépendant de Kesh — garantie de souveraineté numérique.

#### Multi-utilisateurs et sécurité

- **Authentification JWT** avec refresh tokens et rotation automatique (durée d'expiration courte de 15 minutes compensée par rafraîchissement transparent côté frontend).
- **Rôles RBAC** : Administrateur (toutes opérations) et Utilisateur (saisie sans gestion des paramètres système).
- **Isolation multi-tenant stricte** : chaque utilisateur n'accède qu'aux données de sa propre société (`company_id`). Audit complet du scoping effectué Epic 7 (Story 7-1) sur toutes les requêtes API et SQL.
- **Politique de mot de passe** : minimum 12 caractères pour le compte administrateur initial (hardening Story 10-1), configurable pour les autres utilisateurs.
- **Rate limiting** sur la connexion : protection contre les attaques par force brute (5 tentatives échouées par IP avant blocage 30 minutes, paramétrable).
- **Sessions à expiration glissante** : durée d'inactivité de 15 minutes avant déconnexion automatique (paramétrable).

#### Sécurité durcie (Story 10-5)

- **Tokens en cookies HttpOnly + Secure + SameSite=Strict** : les tokens d'authentification (`access_token` JWT + `refresh_token` UUID) sont stockés dans des cookies inaccessibles au JavaScript du navigateur. Élimine la possibilité de vol immédiat des tokens via une faille XSS hypothétique (un script malveillant ne peut ni lire `document.cookie`, ni accéder à `localStorage`). Nouveau endpoint `GET /api/v1/auth/me` permet au frontend de restaurer l'identité utilisateur sans pouvoir décoder le JWT côté JS. Closes Issue [#41 [KF-002]](https://github.com/guycorbaz/kesh/issues/41).
- **Content-Security-Policy défensif** sur les réponses HTML : restreint les sources de scripts, styles, images, connexions ; bloque l'incrustation iframe (`frame-ancestors 'none'`) anti-clickjacking. Défense en profondeur même si l'app reste sans XSS connu.

> **Note pour les administrateurs** : en mode `KESH_TEST_MODE=true` (CI + dev local sur HTTP loopback `127.0.0.1`), le flag `Secure` est désactivé pour permettre les tests E2E sans certificat. En production HTTPS, le flag `Secure` est inconditionnellement actif.

#### Multilingue (FR / DE / IT / EN)

- **Interface utilisateur** disponible en **français, allemand, italien et anglais** — les 4 langues nationales suisses + anglais professionnel.
- **Messages d'erreur** localisés dans toutes les langues.
- **Messages système** (banner DB indisponible, notifications) en 4 langues.

#### Déploiement et opérations

- **Docker Compose** : déploiement standard via une image officielle `gcorbaz/kesh:latest` publiée sur Docker Hub. Documentation complète dans le manuel administrateur.
- **Synology DSM Container Manager** : support natif documenté pour le déploiement sur NAS Synology (DSM 7.2+, modèles x86_64), avec utilisation du Portail des applications DSM comme reverse proxy HTTPS (alternative simple à Nginx/Caddy/Traefik pour LAN-only).
- **Reverse proxy HTTPS** : exemples documentés pour Nginx, Caddy (Let's Encrypt automatique), Traefik (avec firewall applicatif rate-limiting + headers OWASP + plugin CrowdSec optionnel).
- **Healthcheck** `/health` DB-aware (Story 10-3) : retourne `{ status, db, version }` permettant aux orchestrateurs (Docker, Kubernetes, monitoring) de détecter l'état réel de la base de données.
- **Résilience frontend DB inaccessible** (Story 10-3) : si la base de données devient temporairement indisponible (redémarrage MariaDB, panne réseau), l'interface utilisateur reste utilisable en consultation et affiche une bannière "Base de données temporairement indisponible — réessai automatique en cours". Reprise transparente dès que la connexion est rétablie.
- **Migrations DB idempotentes** (Story 10-2) avec protection contre les downgrades silencieux corrupteurs : refus de boot si le binaire Kesh est plus ancien que la version de schéma actuellement déployée.

#### Backup et conformité légale

- **Procédure de backup `mariadb-dump`** documentée avec script bash, rotation 30 jours et hash SHA-256 d'intégrité (conforme OLICo Art. 9 al. 1 lit. b ch. 1).
- **Backup natif sur Synology DSM** (Story 10-4) : documentation complète pour Hyper Backup (incrémental versionné vers cloud ou HDD USB, chiffrement client-side AES-256, rotation Smart Recycle) et Snapshot Replication (Btrfs, recovery point-in-time < 1 minute). Stratégie 3-2-1 illustrée.
- **Procédure de restauration** documentée avec vérification d'intégrité SHA-256, arrêt propre de Kesh, restauration, redémarrage et vérification fonctionnelle.
- **Test de restauration périodique** documenté pour conformité OLICo Art. 10 al. 1 (test annuel obligatoire, procès-verbal conservé 10 ans).
- **Audit-trail des écritures** : chaque modification d'écriture est tracée avec utilisateur, date et snapshot — fournit la garantie d'intégrité requise par OLICo Art. 9 pour les supports modifiables.

#### Documentation

- **Manuel administrateur** (105 pages PDF, français) : installation, configuration, sauvegarde, mise à jour, sécurité, conformité légale suisse, dépannage. Public cible : administrateurs système, responsables DevOps, fiduciaires en self-hosting.
- **Manuel utilisateur** (français) : guide d'utilisation quotidienne pour les comptables et utilisateurs PME.
- **Brochure marketing** (français) : présentation commerciale courte pour découverte du produit.
- Versions DE / IT / EN des manuels prévues v0.2.

### Notes de cette release

- **Production prête, mais pré-1.0** : Kesh v0.1.0 est utilisable en production pour des installations individuelles ou de PME. Le label `0.x` signale que des évolutions sont encore prévues avant v1.0 (notamment TVA Suisse complète, multi-langue des manuels, et fonctionnalités avancées de fiduciaire multi-clients).
- **Pas de migration utilisateur de v0.0.x** : Kesh v0.1.0 est la première version publique. Aucune migration depuis une version antérieure n'est nécessaire ni supportée.
- **Limitations connues v0.1.0** : voir les [issues GitHub](https://github.com/guycorbaz/kesh/issues?q=is%3Aopen+label%3Aknown-failure) avec le label `known-failure`. Aucune ne bloque l'usage productif des fonctionnalités livrées.

### Licence

Kesh est distribué sous [licence EUPL 1.2](https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12) (European Union Public Licence). Cette licence est compatible GPL et permet l'usage commercial, la modification et la redistribution.

---

## Conventions de versionnage

- **MAJOR** (`X.0.0`) : changement incompatible nécessitant action manuelle de l'administrateur (nouvelle migration breaking, changement d'API, refonte UI majeure).
- **MINOR** (`0.X.0`) : nouvelle fonctionnalité rétro-compatible (nouvel epic livré, nouveau module).
- **PATCH** (`0.0.X`) : correction de bug ou amélioration mineure rétro-compatible (sécurité, performance, ergonomie).

Voir la [politique de migration breaking](https://github.com/guycorbaz/kesh/blob/main/CLAUDE.md#migration-breaking-policy) pour le détail technique des migrations DB et de la protection downgrade.
