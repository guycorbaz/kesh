# Release 0.20.0 — ce qui sera fait, et ce que cela touche

**Publication : vendredi 2026-11-27.** Coupure le **jeudi 2026-11-26 au soir** : n'entre que ce qui est clos (boucle de
revue terminée, gates de clôture et E2E complet verts) ; une story non close sort vers une épique ultérieure, **la date ne
bouge pas** (kesh:D7, kesh:D11). Kickoff tenu en autonomie par l'orchestrateur le 2026-11-18, sur les décisions de Guy du
même jour (page « kickoff-0.20.0 » et conversation) ; révisé le même jour sur les réponses de Guy aux points ouverts du
kickoff, puis sur les découpages de la 48-13 (→ 48-15) et de la 49-1 (→ 49-1a à 49-1d). Ce document s'adresse aussi aux contributeurs — en particulier à Beat
et à son agent — pour qu'ils voient d'avance où leurs changements croiseront les nôtres.

> ⚠️ **Une migration** — celle de la 49-1a (types de lignes de la facture client), la 96ᵉ. **Sauvegarde complète
> obligatoire avant la mise à jour du NAS** : base, `.keshbackup` et dossier des documents ; le retour à la 0.19.0 passe
> par elle. Non breaking (colonnes `NOT NULL`, valeurs neutres, `CHECK` assouplis par type) :
> `kesh_version_min_required` reste à 0.17.6.

## Décisions qui fondent le plan

| Réf. | Décision | Date |
|---|---|---|
| kesh:D11 | Une épique non terminée à la release se clôt avec elle ; ses stories non faites passent à une épique ultérieure. L'Epic 48 est le réservoir dont ce kickoff tire le périmètre. | 2026-11-18 |
| kesh:D13 | Une release publiée avant son vendredi compte pour la semaine : la 0.19.0 (mercredi 2026-11-18) est celle de la semaine ; **la 0.20.0 garde le vendredi 2026-11-27**. | 2026-11-18 |
| kesh:D14 | La 0.20.0 s'attaque **principalement aux bugs**, avec une ou deux fonctionnalités. | 2026-11-18 |
| Guy, 2026-11-18 | **Epic 48** devient l'épique corrective de la release, **sept stories** par dérogation à la borne de six (près de dix jours de travail, « il y a deux jours de plus »), **huit** le même jour avec la 48-15 (même motif, et l'élargissement voulu par Guy pour faire baisser les bugs) ; 48-3 à 48-9 hors 0.20.0. | 2026-11-18 |
| Guy, 2026-11-18 | **Epic 49** — types de lignes de la facture client (#1242), trois stories au kickoff, toutes visées pour la 0.20.0 ; la 49-1 porte la migration. | 2026-11-18 |
| Guy, 2026-11-18 | **48-13 découpée** à sa validation P2 : la **48-15-troncatures-silencieuses** (#1331 à #1334, ajoutées à la release pour diminuer les bugs, et trois champs du PDF) en naît, avec **dérogation à la règle de splitting** (sept domaines, un même geste mécanique en deux lots) ; elle part de la 48-13 fusionnée et ne bloque pas l'Epic 49. Sites voisins hors périmètre : #1335 à #1340, sans milestone. | 2026-11-18 |
| Orchestrateur, 2026-11-18 | **49-1 découpée** (splitting préventif, neuf modules) en 49-1a (migration, modèle, API gardée), 49-1b (PDF), 49-1c (écran), 49-1d (recette de montée) ; **la 49-1a ne sort jamais sans la 49-1d**, la 49-1c pas sans la 49-1b ; trois types stockés (`priced`, `comment`, `heading`), pas de sous-total (#1330). L'Epic 49 compte six stories. | 2026-11-18 |
| Guy, 2026-11-18 | **Sous-totaux repoussés** à une release ultérieure : Beat tient une comptabilité assujettie à la TVA, un sous-total doit être juste en HT/TTC et par taux, ou attendre. La 49-3 ne porte plus que les **titres de section** (clé renommée `49-3-titres-de-section`). La migration de la 49-1 pose les types produit, sans produit, commentaire et titre ; le type sous-total n'y entre que si la fiche de la 49-1 établit qu'il ne préjuge pas de son sens TVA, sinon il viendra avec sa propre migration. Issue à ouvrir par l'orchestrateur, « sous-totaux de section », milestone « Plus tard ». | 2026-11-18 |
| Guy, 2026-11-18 | **#1234 retirée** de la 48-13 et de la 0.20.0, renvoyée à « Plus tard » (latente tant que Kesh ne renseigne pas `billing_information`). | 2026-11-18 |
| Guy, 2026-11-18 | **La 48-13 se fusionne avant** les stories de l'Epic 49 qui touchent `InvoiceForm.svelte` (petite, sur le chemin de la première facture de fin novembre). | 2026-11-18 |
| Guy, 2026-11-18 | Story de solde : **#1263, #1262, #1261, #1255** au milestone v0.20.0 ; milestone **v0.21.0** créé (vendredi 2026-12-04), #1236 y est. | 2026-11-18 |
| Guy, 2026-11-18 (complément) | **Des factures réelles existent déjà** en production (remises sur papier à un locataire) : la migration de la 49-1 les conserve à l'identique, une facture émise garde son PDF, recette de montée avec factures réelles simulées comme gate de la release ; les fiches 48-1, 48-2 et 48-13 citent ce cas dans leurs AC. | 2026-11-18 |
| Orchestrateur, 2026-11-18 (délégation de Guy) | Migration **non breaking** : colonnes `NOT NULL`, valeurs neutres pour les lignes non chiffrées, `CHECK` assouplis par type, `credit_note_lines` comprise ; **splitting préventif** de la 49-1 (et de la 48-12, de la 48-13 si besoin) jugé à la fiche, selon la règle ; **charge de la voie B** surveillée par l'orchestrateur. | 2026-11-18 |
| Guy, 2026-11-18 | **v0200-menu-sous-groupes** (#1253) : l'amélioration de la release (une par release, décision de Guy du 2026-11-17), hors épique. | 2026-11-18 |
| Guy, 2026-11-18 | **Une seule migration** en 0.20.0, celle de la 49-1 (49-1a depuis le découpage). | 2026-11-18 |
| Guy, 2026-11-18 | **Deux épiques en parallèle** (48 et 49), sur les deux voies de la station, pour la vitesse. | 2026-11-18 |
| Guy, 2026-11-18 | **Correctifs de Beat** : la PR #1327 et trois correctifs annoncés entrent s'ils sont fusionnés avant la coupure, après revue d'intégration et CI de la copie ; #1327 se fusionne avant le lancement de la 48-11. | 2026-11-18 |
| Guy, 2026-11-18 | Ordre de sortie : la story de solde (48-14) puis la 48-2 sortent les premières ; **48-11 et 48-13 sont prioritaires**. | 2026-11-18 |

## Base de départ

- `main` au tag **v0.19.0** (`773df80da`). Production (NAS) en 0.19.0 depuis le 2026-11-18. **95 migrations** ;
  `kesh_version_min_required` = 0.17.6.
- La branche `chore/apres-release-0.19.0` (bilan, réponses de Guy, Epic 48 réservoir, `CLAUDE.md`) et ce kickoff
  (`chore/kickoff-0.20.0`, qui en part) ne touchent que la documentation et le registre ; ils se fusionnent avant les
  épiques.
- Branches : `epic/48-corrections-0-20-0` et `epic/49-types-de-lignes-facture`, parties de `main` ; les stories en
  `story/…`, fusionnées dans leur branche d'épique dès leur clôture, puis poussées. La story hors épique
  `v0200-menu-sous-groupes` part de `main` sur sa branche `story/…`.
- **L'annonce du plan de la 0.19.0 est retirée** : il annonçait pour la 0.20.0 une migration de la **note interne** (#1158,
  #268). Elle n'y est pas — #1158, #268 sortent du milestone (voir plus bas).

## Contenu

### Epic 48 — Corrections de la 0.20.0 (`epic-48-corrections-0-20-0.md`)

| Rang | Story | Issues | Migration | Si le temps manque |
|---|---|---|---|---|
| 1 | 48-13 — Avant la vraie facture : client écrasé par un chargement tardif, adresse tronquée en silence, nom de société non contrôlé à l'assistant ; **se fusionne avant les stories de l'Epic 49** | #1233, #1243, #1226 | non | **prioritaire** |
| 2 | 48-11 — Les montants à deux décimales sont arrondis au centime, non tronqués (exports CSV, rapports CSV et PDF) ; **après la fusion de #1327** | #1245 (P2) | non | **prioritaire** |
| 3 | 48-12 — Plus de doublon ni d'écrasement après une réponse perdue (factures fournisseur, comptes bancaires) | #975, #984, #985, #986 (P2) | non | sort après la 48-2 |
| 4 | 48-10 — « 1.250 » refusé au prix d'article, aux frais de rappel, au montant minimal, aux filtres des écritures | #1173, #1175, #1176, #1177 | non | sort après la 48-2 |
| 5 | 48-1 — Changer le compte analytique d'une facture fournisseur, même payée | #1167 (refs) | non | sort après la 48-2 |
| 6 | 48-2 — Idem, écriture manuelle ; #1167 se ferme avec elle | #1167 | non | **sort la deuxième** |
| 7 | 48-15 — Un texte trop long est refusé en le disant, non coupé en silence : adresse du client, pays du fournisseur, e-mail de la société, prénom et nom ; nom d'article, numéro de client, conditions de paiement (PDF) ; **après la 48-13** ; dérogation à la règle de splitting | #1331, #1332, #1333, #1334 | non | sort **avant** la 48-11 et la 48-13 |
| 8 | 48-14 — Story de solde : dette des revues de la 0.19.0 (règlement groupé), **plafond de quatre issues** | #1263, #1262, #1261, #1255 | non | **sort la première** |

### Epic 49 — Types de lignes de la facture client (`epic-49-types-de-lignes-facture.md`) — porte la migration

| Rang | Story | Issues | Migration | Si le temps manque |
|---|---|---|---|---|
| 1 | 49-1a — La migration (trois types stockés : `priced`, `comment`, `heading` ; contraintes par type sur `invoice_lines` et `credit_note_lines`), le modèle, les dépôts, la reprise des avoirs ; **API gardée** (seule `priced` acceptée) ; manuel d'administration « Pour les intégrations » | #1242 (P2) | **oui** | **ne sort jamais sans la 49-1d** ; leur sortie = la release n'a plus de migration |
| 2 | 49-1d — Recette de montée (factures réelles simulées, `credit_note_lines` dans l'empreinte), section « Passer à la 0.20.0 » du manuel d'administration | #1242 | non | liée à la 49-1a |
| 3 | 49-1b — Rendu PDF d'un commentaire, facture et avoir ; test avant / après | #1242 | non | sort la dernière de l'épique, après la 49-1c |
| 4 | 49-1c — L'écran : sélecteur de type, ligne de commentaire ; ouvre `comment` ; `verify_vat_rates_against_db` | #1242 | non | ne sort pas sans la 49-1b ; sort après la 49-2 |
| 5 | 49-2 — Réordonner les lignes d'un brouillon (↑ / ↓ ; glisser-déposer si sûr) | #1242 | non | sort après la 49-3 |
| 6 | 49-3 — Titres de section (sans montant), rendu PDF, reprise dans les avoirs ; ouvre `heading` ; les **sous-totaux sont repoussés** (#1330) | #1242 | non (posée par la 49-1a) | sort la première de l'épique |

Échéance de Guy : **fin novembre**, pour ses vraies factures — la 0.20.0 est la dernière release avant elle.

### Hors épique — l'amélioration de la release

| Story | Issue | Migration | Note |
|---|---|---|---|
| v0200-menu-sous-groupes — le menu Administration en sous-groupes (Comptabilité, Banque, Facturation, Système) | #1253 (P4) | non | frontend seul ; change les chemins décrits par les **quatre manuels** : à fusionner **en dernier** |

### Ordre de sortie à la coupure, toutes épiques

Si le temps manque, dans l'ordre : **48-14**, **48-2**, puis selon l'avancement 49-3, 49-2, 49-1c, 48-1, 48-10, 48-12,
v0200-menu-sous-groupes, 49-1b ; la **48-15** sort avant la 48-11 et la 48-13 ; **48-11 et 48-13 restent les dernières**.
La 49-1a ne sort qu'avec la 49-1d, et emporte la migration ; la 49-1c ne reste pas sans la 49-1b. Le Project Lead garde
le dernier mot ; l'orchestrateur ne touche jamais aux épiques d'un contributeur.

## Contraintes de production — des factures réelles existent déjà

Guy a déjà créé de vraies factures client dans Kesh (NAS) et les a remises sur papier à un locataire. D'où, pour toute la
release (détail : `epic-49-types-de-lignes-facture.md`, § *Contraintes de production*) :

- **la migration de la 49-1a conserve à l'identique chaque facture et chaque avoir existants** : chaque ligne existante
  reçoit le type chiffré unique `priced`, rendu comme aujourd'hui (aucun lien au catalogue n'est gardé en base : « produit »
  et « sans produit » ne diffèrent qu'à l'écran ; aucune origine n'est devinée) ; totaux, TVA, écritures et numéros inchangés ; triage
  P7 au registre `POST_RESTORE_BACKFILLS` ou `EXEMPT_MIGRATIONS`, avec justification ; un import d'une sauvegarde 0.19.0
  donne les mêmes factures ;
- **une facture déjà émise garde son PDF** : le PDF archivé (25-6-b) reste la référence ; une facture émise avant la
  0.20.0 se réimprime à l'identique, ou son PDF archivé est servi tel quel ; la fiche le prouve par un test de rendu
  avant / après d'une facture validée de la 0.19.0 ;
- **48-1, 48-2, 48-13 et 48-15** s'appliquent à des factures et écritures réelles : leurs fiches citent ce cas dans leurs AC.

## Gates de la release

Outre les gates habituels (gate complet, E2E complet, `gate-ci`, revue du code d'intégration, CI) :

- **Recette de montée avec des factures réelles simulées — obligatoire** : base peuplée par la **0.19.0** (dernière
  image publiée) avec des factures client **validées, payées et non payées**, **un avoir**, **une facture réglée en
  groupe** ; montée en 0.20.0 ; comparaison, facture par facture, des **totaux**, des **écritures** et du **PDF**.
  Installation jetable seulement, jamais la base du NAS (`scripts/recette-montee.sh`).
- **Import d'une sauvegarde 0.19.0** de cette même base : mêmes factures après import qu'après montée.

## Migration annoncée

Une seule, portée par la **49-1a** : une colonne de type sur `invoice_lines` et `credit_note_lines`, et des contraintes par
type à la place des `CHECK` actuels (`quantity > 0`, description non vide, montant non négatif). Trois types stockés :
`priced` (produit ou sans produit), `comment`, `heading` ; pas de sous-total (#1330). **Non breaking** (colonnes `NOT NULL`, valeurs
neutres) ;
horodatée après `20261118000001_grouped_settlements` ; audit d'idempotence (P5), couplage positionnel (P6), triage P7 si
elle écrit des données, schéma de test, `.keshbackup` et export de souveraineté. Le détail est au fichier de l'Epic 49.
**Annoncée au projet GitHub** (champ de #1242) et en tête des notes de version.

Pour les contributeurs : une migration qui partirait de `main` pendant la 0.20.0 porte un horodatage **postérieur** à
celle de la 49-1a et s'annonce d'avance ; Guy a décidé qu'aucune autre n'entre dans la 0.20.0.

## Voies et parallélisme

**Deux épiques internes en parallèle** (48 et 49) : c'est un **écart assumé** à « une épique en cours par développeur »
(kesh:D6), décidé par Guy le 2026-11-18 pour la vitesse. La station tient deux voies (verrous base/Rust et frontend
séparés, `mem-guard` à deux exécutions de 10G) :

- **voie A — Epic 49** : la 49-1a d'abord (migration, échéance de Guy), puis 49-1b et 49-1d, 49-1c, 49-2 et 49-3 ;
- **voie B — Epic 48** : 48-13, 48-11 (dès #1327), 48-15 (dès la 48-13 fusionnée), 48-12, 48-10, 48-1, 48-2, 48-14 ;
- **v0200-menu-sous-groupes** (frontend seul) s'intercale sur les créneaux frontend libres, et fusionne en dernier.

Chaque story a son worktree et ses bases `kesh_<clé>` ; aucun agent ne redémarre le conteneur MariaDB.

## Correctifs de Beat (contributeur)

Ils entrent dans la 0.20.0 **s'ils sont fusionnés dans `main` avant la coupure**, après revue d'intégration et CI de la
copie ; aucun ne porte de migration :

| Correctif | État au 2026-11-18 | Fichiers | Croise |
|---|---|---|---|
| PR **#1327** — mise en page des PDF de rapports (montants alignés à droite, titre accentué, en-têtes de devise) | ouverte ; CI sur la copie **#1329** | `kesh-report/src/pdf.rs`, `kesh-report/Cargo.toml`, `kesh-qrbill/src/largeurs.rs`, `kesh-qrbill/src/lib.rs`, catalogues (4 locales), `user-manual.tex`, `Cargo.lock`, `sprint-status.yaml`, `architecture.md` | **48-11** (même `pdf.rs` : se fusionne **avant** son lancement) ; 49-1 et 49-3 (`kesh-qrbill`) |
| Cours AFC des monnaies retirées | annoncé | à lire à sa PR (cours de change) | — |
| Ligne « Résultat reporté » à zéro | annoncé | à lire à sa PR (rapports) | 48-11 si `kesh-report` |
| Parent d'un compte modifiable | annoncé | à lire à sa PR (plan comptable) | — |

## Milestone v0.20.0

Fait par l'orchestrateur le 2026-11-18, sur décision de Guy. **Vingt-trois issues** (contrôlé le 2026-11-18 au soir :
vingt-trois ouvertes) : #1242, #1245, #1173, #1175, #1176, #1177, #975, #984, #985, #986, #1233, #1243, #1226, #1253,
#1167 ; les quatre de la story de solde, #1263, #1262, #1261, #1255 ; et les quatre de la 48-15, #1331, #1332, #1333,
#1334, ajoutées par Guy pour diminuer les bugs. Hors milestone : #1330 (sous-totaux, « Plus tard ») et #1335 à #1340
(sites voisins de la 48-15, sans milestone). **En sortent** : #1234 (→ « Plus tard »), #1236 (état « envoyée » → milestone
**v0.21.0**, créé pour le vendredi 2026-12-04, fonctionnalité principale envisagée), #1158 et #268 (note interne), #944
(QR-facture, suite), #1125 (tests figés en 2026).

## Décompte au kickoff et objectif

Recompté le 2026-11-18 par les commandes du `CLAUDE.md` :

| | Tag v0.19.0 | Kickoff (2026-11-18) | Objectif au tag v0.20.0 (non barrière) |
|---|---|---|---|
| Issues ouvertes | 496 | **496** | moins de 496 |
| Bugs (`bug` ou `known-failure`) | 247 | **247** | moins de 247 |
| P1 ouvertes | 14 | **14** (dont 11 `bug`/`known-failure`) | — |
| *Le soir du 2026-11-18* | | *507 issues, 257 bugs, 14 P1* | *les fiches de la release ont ouvert #1330 à #1340* |

Hors les **56 issues masquées** du compte de l'agent de Beat (#1266–#1271, #1277–#1326), non comptées ; décision de Guy :
attendre leur rétablissement jusqu'au 2026-11-25, sinon recréation depuis le compte de Beat, et correction du décompte au
tag par une annotation datée.

La release vise **dix-neuf bugs** (`bug` ou `known-failure`), tous au milestone : douze dans les stories 48-10 à 48-13
(#1245, #1173, #1175, #1176, #1177, #975, #984, #985, #986, #1233, #1243, #1226), quatre dans la 48-15 (#1331, #1332,
#1333, #1334) et trois dans la story de solde (#1261, #1262, #1263). Aucune des
quatorze P1 ouvertes (dont les onze de la TVA en report assumé) n'est dans le périmètre : celles qui restent se nomment à
la rétrospective avec leur raison (`CLAUDE.md` § *Priorités des défauts*).

## Fichiers communs

| Fichier | Qui | Règle |
|---|---|---|
| `frontend/src/lib/components/invoices/InvoiceForm.svelte` | 48-13 (#1233) ; 49-1c, 49-2, 49-3 | **la 48-13 se fusionne d'abord** (arbitrage de Guy du 2026-11-18) ; les stories de l'Epic 49 qui touchent le fichier partent ensuite de la branche à jour |
| `crates/kesh-qrbill/src/pdf.rs`, `types.rs` | 49-1b, 49-3 ; #1327 (`largeurs.rs`, `lib.rs`) | le second arrivé résout |
| `crates/kesh-report/src/pdf.rs` | 48-11 ; #1327 | 48-11 après #1327 |
| `crates/kesh-db/src/repositories/supplier_invoices.rs`, `kesh-api/src/routes/supplier_invoices.rs` | 48-1 ; 48-12 (si clé d'idempotence) | même épique ; le second arrivé résout |
| `routes/(app)/settings/+page.svelte` | 48-13 (#1243) ; 48-15 (#1333) ; v0200-menu-sous-groupes (peut-être) | blocs distincts ; la 48-15 après la 48-13 |
| `crates/kesh-api/src/routes/onboarding.rs` (`set_coordinates`) | 48-13 (#1226) ; 48-15 (#1334) | la 48-15 part de la 48-13 fusionnée |
| `routes/(app)/contacts/+page.svelte`, `routes/(app)/products/+page.svelte` | 48-15 ; 48-10 (`products`) | blocs distincts ; le second arrivé résout |
| `scripts/recette-montee.sh`, manuel d'administration ×4 | 49-1a, 49-1d | la 49-1d complète ce que pose la 49-1a |
| catalogues i18n (4 locales), `sprint-status.yaml` | tous | chaque épique dans son bloc ; conflits résolus en gardant les deux côtés |
| `sitesTotal`, totaux de l'audit des migrations, schéma de test | 48-*, 49-* | recomptés après chaque fusion |
| manuels `.tex` (4 langues) | tous ; le menu change les chemins | le menu fusionne en dernier ; PDF régénérés à la release seulement |
| `CHANGELOG.md` | — | assemblé à l'intégration depuis les *Notes de version* des fiches |
| `Cargo.lock` | #1327 ; toute story qui change une dépendance | régénéré après fusion |

## Calendrier

| Jour | Quoi |
|---|---|
| mer. 2026-11-18 | kickoff (ce document) ; fiches 48-13, 48-15, 48-11, 49-1a à écrire en premier (48-13, 48-15, 49-1a en validation, P3) |
| jeu. 19 → mer. 25 | développement sur les deux voies ; chaque story close se fusionne dans sa branche d'épique et se pousse |
| mer. 2026-11-25 | échéance des issues masquées de Beat (décision de Guy) |
| **jeu. 2026-11-26 au soir** | **coupure** : n'entre que ce qui est clos ; sortie dans l'ordre ci-dessus |
| **ven. 2026-11-27** | intégration (fusions, gates, recette de montée depuis l'image 0.19.0 sur une base peuplée de factures réelles simulées, revue du code d'intégration, PR, CI) ; accord de fusion de Guy vers midi ; tag et image ; NAS par Guy, **après sauvegarde complète** |

## Points ouverts du kickoff — réglés le 2026-11-18

Les huit points ouverts du kickoff sont réglés ; il n'en reste aucun.

1. **`InvoiceForm.svelte` partagé entre la 48-13 et l'Epic 49** — Guy : la 48-13 se fusionne d'abord ; les stories de
   l'Epic 49 qui touchent ce fichier partent ensuite de la branche à jour. `kesh-qrbill` n'est plus partagé (#1234 retirée).
2. **Splitting préventif** — orchestrateur : la règle s'applique à la fiche. Appliquée : la 49-1 est découpée en 49-1a à
   49-1d ; la 48-13 a été coupée en 48-13 et 48-15, celle-ci avec dérogation (décision de Guy) ; la 48-12 reste à juger.
3. **Migration** — orchestrateur : non breaking, colonnes `NOT NULL`, valeurs neutres pour les lignes non chiffrées,
   `CHECK` assouplis par type, `credit_note_lines` comprise ; `kesh_version_min_required` reste à 0.17.6.
4. **Sous-totaux** — Guy : repoussés à une release ultérieure (justesse HT/TTC et par taux, Beat étant assujetti) ; la
   49-3 devient `49-3-titres-de-section` ; issue #1330 (sous-totaux de section, milestone « Plus tard »).
5. **Issues de la story de solde** — Guy : #1263, #1262, #1261, #1255 sont au milestone v0.20.0.
6. **Milestone v0.21.0** — créé (vendredi 2026-12-04) ; #1236 y est.
7. **Charge de la voie B** — surveillée par l'orchestrateur ; l'ordre de sortie s'applique à la coupure.
8. **#1234** — Guy : retirée de la 48-13 et de la 0.20.0, renvoyée à « Plus tard ».
