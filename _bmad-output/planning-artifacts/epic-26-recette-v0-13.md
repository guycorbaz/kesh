# Epic 26 — Recette de la v0.13.0 : l'import CSV d'UBS tel quel, et le nom des comptes

**Kickoff : 2026-10-10.** La v0.13.0 est en fonction sur le NAS depuis le 2026-10-10, sur une
base vierge. Guy y commence sa comptabilité personnelle, exercice ouvert au 1er janvier 2026, et
c'est cette recette qui a produit les issues de cet epic. **Cible : v0.13.1.**

⚠️ **Contrepartie BMAD des issues de la recette** — les issues tiennent le détail, cet epic tient
le périmètre et l'ordre. Il ne se clôt qu'avec les six issues ci-dessous fermées, à l'exception
de #621, qui reste ouverte pour la lecture du préambule (cf. 26-1).

## Le constat qui fonde l'epic

UBS ne propose à Guy que deux formats d'extrait : MT940 et CSV. Kesh n'importe pas le MT940
(#618, reporté), si bien que **le CSV est la seule voie d'entrée de ses mouvements UBS**. Or le
fichier d'UBS, tel qu'il sort de l'e-banking, n'est pas importable :

- son préambule compte **neuf lignes** avant la ligne de titres (huit lignes d'informations et une
  ligne vide), alors que le profil plafonne le nombre de lignes à ignorer à **5** (#621) ;
- en mode débit et crédit séparés, Kesh **inverse le signe de la colonne Débit** ; UBS y écrit déjà
  ses débits en négatif, si bien que **chaque dépense devient un encaissement** (#620, P1 : erreur
  comptable).

Guy refuse le contournement par un fichier converti : *« je veux tester un système standard »*.
L'import doit accepter le fichier d'UBS sans retouche.

## Décisions du Project Lead (2026-10-10)

1. **Le sens des montants est un réglage du profil**, jamais une déduction de Kesh (#620). Le
   profil dit explicitement comment la banque écrit ses débits.
2. **Le nombre de lignes à ignorer reste un réglage**, plafond relevé (#621). La détection
   automatique de la ligne de titres est écartée.
3. **La lecture de l'IBAN et des soldes du préambule est reportée** — #621 reste ouverte pour elle.
4. **La description d'un compte bancaire est facultative** (#622). Sans elle, l'affichage reste
   celui d'aujourd'hui.
5. **La v0.13.1 embarque aussi** les deux défauts du manuel (#616, #617) et la garde de la
   restauration Synology (#619). Le MT940 (#618) et la sauvegarde avec fichiers (#503) vont à une
   version suivante.

## Les stories, et pourquoi dans cet ordre

### 26-1 — Importer le CSV d'UBS tel quel ([#620], [#621])

**Première, parce que Guy l'attend pour poursuivre sa recette**, et parce qu'elle porte le seul
P1. Deux réglages du profil bancaire :

- **le sens des montants** : en mode débit et crédit séparés, dire si la colonne Débit porte des
  montants positifs (Kesh les rend négatifs, comportement actuel) ou déjà négatifs (Kesh les garde).
  La spec tranche si le réglage vaut aussi en mode colonne unique (banque qui écrirait ses dépenses
  en positif) ;
- **le nombre de lignes à ignorer** : plafond relevé au-delà de 5 ; la spec fixe la nouvelle borne
  et dit si une ligne vide compte (le crate `csv` saute les lignes vides).

Points que la spec doit trancher, et qui arment les garde-fous de la § *Migration breaking
policy* :

- le plafond vit **aussi** dans la base (`CHECK (header_row_count <= 5)`, migration
  `20260505000001_bank_profiles.sql`) : le relever est un changement de contrainte ;
- `ColumnMapping` est sérialisé en JSON avec `deny_unknown_fields` : y loger le nouveau réglage
  rendrait le profil **illisible par une 0.13.0**. Colonne dédiée ou champ JSON, la spec choisit et
  justifie au regard de P1 à P8, de l'export `.keshbackup` et de l'export CSV des tables ;
- **critère d'acceptation sur le fichier d'UBS réel, anonymisé**, versionné comme fixture : 196
  transactions, somme des montants égale au solde final moins le solde initial (−22 458.62).

PR : `closes #620`, `refs #621` (#621 reste ouverte pour le préambule).

### 26-2 — Une courte description pour chaque compte bancaire ([#622])

« UBS — Personnel », « UBS — Épargne » : distinguer deux comptes de la même banque autrement que
par l'IBAN. Facultative ; sans elle, l'affichage actuel. **Une vingtaine d'écrans** affichent un
compte bancaire (import, réconciliation, règlement de facture, lots de paiement, accueil…) : la
spec en dresse l'inventaire **clos** — sites qui affichent `bankName` sans la description — et
centralise le libellé dans une seule fonction. Elle tranche longueur, unicité, présence dans
l'export et le `.keshbackup`, et la migration (`ADD COLUMN` nullable : non breaking, triage P5/P7).
Candidate au découpage si l'inventaire dépasse cinq modules.

PR : `closes #622`.

### 26-3 — Le manuel et la restauration Synology ([#616], [#617], [#619])

- #616 : le manuel utilisateur fait cocher une case « Compte de banque » dans le plan comptable,
  qui n'existe pas ; le lien se fait par Administration → Comptes bancaires ;
- #617 : le manuel dit le CAMT.053 « fourni par toutes les banques suisses » — faux ;
- #619 : `kesh-restore.sh` repère Kesh par le nom de service `kesh-api` et sa garde **échoue
  ouverte** si le service est renommé ; le manuel d'administration suppose le réseau `frontend`.

Indépendante des deux autres ; peut avancer en parallèle.

PR : `closes #616`, `closes #617`, `closes #619`.

### Release v0.13.1

Après les trois stories : `scripts/prepare-release.sh`, macros des manuels, CHANGELOG, tag annoté,
vérifications Docker Hub. Puis mise à jour du NAS de Guy (`gcorbaz/kesh:0.13.1`).

[#616]: https://github.com/guycorbaz/kesh/issues/616
[#617]: https://github.com/guycorbaz/kesh/issues/617
[#619]: https://github.com/guycorbaz/kesh/issues/619
[#620]: https://github.com/guycorbaz/kesh/issues/620
[#621]: https://github.com/guycorbaz/kesh/issues/621
[#622]: https://github.com/guycorbaz/kesh/issues/622
