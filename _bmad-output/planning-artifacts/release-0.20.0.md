# Release 0.20.0 — ce qui sera fait, et ce que cela touche

**Publication : vendredi 2026-11-27.** Coupure le **jeudi 2026-11-26 au soir** : n'entre que ce qui est clos (boucle de
revue terminée, gates de clôture et E2E complet verts) ; une story non close sort vers une épique ultérieure, **la date ne
bouge pas** (kesh:D7, kesh:D11). Kickoff tenu en autonomie par l'orchestrateur le 2026-11-18, sur les décisions de Guy du
même jour (page « kickoff-0.20.0 » et conversation). Ce document s'adresse aussi aux contributeurs — en particulier à Beat
et à son agent — pour qu'ils voient d'avance où leurs changements croiseront les nôtres.

> ⚠️ **Une migration** — celle de la 49-1 (types de lignes de la facture client), la 96ᵉ. **Sauvegarde complète
> obligatoire avant la mise à jour du NAS** : base, `.keshbackup` et dossier des documents ; le retour à la 0.19.0 passe
> par elle. Non breaking visée : `kesh_version_min_required` resterait à 0.17.6.

## Décisions qui fondent le plan

| Réf. | Décision | Date |
|---|---|---|
| kesh:D11 | Une épique non terminée à la release se clôt avec elle ; ses stories non faites passent à une épique ultérieure. L'Epic 48 est le réservoir dont ce kickoff tire le périmètre. | 2026-11-18 |
| kesh:D13 | Une release publiée avant son vendredi compte pour la semaine : la 0.19.0 (mercredi 2026-11-18) est celle de la semaine ; **la 0.20.0 garde le vendredi 2026-11-27**. | 2026-11-18 |
| kesh:D14 | La 0.20.0 s'attaque **principalement aux bugs**, avec une ou deux fonctionnalités. | 2026-11-18 |
| Guy, 2026-11-18 | **Epic 48** devient l'épique corrective de la release, **sept stories** par dérogation à la borne de six (près de dix jours de travail, « il y a deux jours de plus ») ; 48-3 à 48-9 hors 0.20.0. | 2026-11-18 |
| Guy, 2026-11-18 | **Epic 49** — types de lignes de la facture client (#1242), trois stories, toutes visées pour la 0.20.0 ; la 49-1 porte la migration, qui pose d'emblée les cinq types. | 2026-11-18 |
| Guy, 2026-11-18 | **v0200-menu-sous-groupes** (#1253) : l'amélioration de la release (une par release, décision de Guy du 2026-11-17), hors épique. | 2026-11-18 |
| Guy, 2026-11-18 | **Une seule migration** en 0.20.0, celle de la 49-1. | 2026-11-18 |
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
| 1 | 48-13 — Avant la vraie facture : client écrasé par un chargement tardif, adresse tronquée en silence, nom de société non contrôlé à l'assistant, informations de facturation de la QR-facture | #1233, #1243, #1226, #1234 | non | **prioritaire** |
| 2 | 48-11 — Les montants à deux décimales sont arrondis au centime, non tronqués (exports CSV, rapports CSV et PDF) ; **après la fusion de #1327** | #1245 (P2) | non | **prioritaire** |
| 3 | 48-12 — Plus de doublon ni d'écrasement après une réponse perdue (factures fournisseur, comptes bancaires) | #975, #984, #985, #986 (P2) | non | sort après la 48-2 |
| 4 | 48-10 — « 1.250 » refusé au prix d'article, aux frais de rappel, au montant minimal, aux filtres des écritures | #1173, #1175, #1176, #1177 | non | sort après la 48-2 |
| 5 | 48-1 — Changer le compte analytique d'une facture fournisseur, même payée | #1167 (refs) | non | sort après la 48-2 |
| 6 | 48-2 — Idem, écriture manuelle ; #1167 se ferme avec elle | #1167 | non | **sort la deuxième** |
| 7 | 48-14 — Story de solde : dette des revues de la 0.19.0 (règlement groupé), **au plus quatre issues** parmi #1255 à #1265 | à arrêter à la fiche | non | **sort la première** |

### Epic 49 — Types de lignes de la facture client (`epic-49-types-de-lignes-facture.md`) — porte la migration

| Rang | Story | Issues | Migration | Si le temps manque |
|---|---|---|---|---|
| 1 | 49-1 — Types produit, sans produit, commentaire ; la migration des cinq types (titre et sous-total compris), contraintes par type sur `invoice_lines` et `credit_note_lines` ; rendu PDF ; avoirs qui reprennent commentaires et titres | #1242 (P2) | **oui** | sortie = la release n'a plus de migration |
| 2 | 49-2 — Réordonner les lignes d'un brouillon (↑ / ↓ ; glisser-déposer si sûr) | #1242 | non | sort après la 49-3 |
| 3 | 49-3 — Titres de section et sous-totaux, rendu PDF | #1242 | non (posée par la 49-1) | sort la première de l'épique |

Échéance de Guy : **fin novembre**, pour ses vraies factures — la 0.20.0 est la dernière release avant elle.

### Hors épique — l'amélioration de la release

| Story | Issue | Migration | Note |
|---|---|---|---|
| v0200-menu-sous-groupes — le menu Administration en sous-groupes (Comptabilité, Banque, Facturation, Système) | #1253 (P4) | non | frontend seul ; change les chemins décrits par les **quatre manuels** : à fusionner **en dernier** |

### Ordre de sortie à la coupure, toutes épiques

Si le temps manque, dans l'ordre : **48-14**, **48-2**, puis selon l'avancement 49-3, 49-2, 48-1, 48-10, 48-12,
v0200-menu-sous-groupes ; **48-11 et 48-13 restent les dernières**. La 49-1 sort avec sa migration. Le Project Lead garde
le dernier mot ; l'orchestrateur ne touche jamais aux épiques d'un contributeur.

## Migration annoncée

Une seule, portée par la **49-1** : une colonne de type sur `invoice_lines` et `credit_note_lines`, et des contraintes par
type à la place des `CHECK` actuels (`quantity > 0`, description non vide, montant non négatif). **Non breaking visée** ;
horodatée après `20261118000001_grouped_settlements` ; audit d'idempotence (P5), couplage positionnel (P6), triage P7 si
elle écrit des données, schéma de test, `.keshbackup` et export de souveraineté. Le détail est au fichier de l'Epic 49.
**Annoncée au projet GitHub** (champ de #1242) et en tête des notes de version.

Pour les contributeurs : une migration qui partirait de `main` pendant la 0.20.0 porte un horodatage **postérieur** à
celle de la 49-1 et s'annonce d'avance ; Guy a décidé qu'aucune autre n'entre dans la 0.20.0.

## Voies et parallélisme

**Deux épiques internes en parallèle** (48 et 49) : c'est un **écart assumé** à « une épique en cours par développeur »
(kesh:D6), décidé par Guy le 2026-11-18 pour la vitesse. La station tient deux voies (verrous base/Rust et frontend
séparés, `mem-guard` à deux exécutions de 10G) :

- **voie A — Epic 49** : la 49-1 d'abord (migration, échéance de Guy), puis 49-2 et 49-3 ;
- **voie B — Epic 48** : 48-13, 48-11 (dès #1327), 48-12, 48-10, 48-1, 48-2, 48-14 ;
- **v0200-menu-sous-groupes** (frontend seul) s'intercale sur les créneaux frontend libres, et fusionne en dernier.

Chaque story a son worktree et ses bases `kesh_<clé>` ; aucun agent ne redémarre le conteneur MariaDB.

## Correctifs de Beat (contributeur)

Ils entrent dans la 0.20.0 **s'ils sont fusionnés dans `main` avant la coupure**, après revue d'intégration et CI de la
copie ; aucun ne porte de migration :

| Correctif | État au 2026-11-18 | Fichiers | Croise |
|---|---|---|---|
| PR **#1327** — mise en page des PDF de rapports (montants alignés à droite, titre accentué, en-têtes de devise) | ouverte ; CI sur la copie **#1329** | `kesh-report/src/pdf.rs`, `kesh-report/Cargo.toml`, `kesh-qrbill/src/largeurs.rs`, `kesh-qrbill/src/lib.rs`, catalogues (4 locales), `user-manual.tex`, `Cargo.lock`, `sprint-status.yaml`, `architecture.md` | **48-11** (même `pdf.rs` : se fusionne **avant** son lancement) ; 49-1 et 48-13 (`kesh-qrbill`) |
| Cours AFC des monnaies retirées | annoncé | à lire à sa PR (cours de change) | — |
| Ligne « Résultat reporté » à zéro | annoncé | à lire à sa PR (rapports) | 48-11 si `kesh-report` |
| Parent d'un compte modifiable | annoncé | à lire à sa PR (plan comptable) | — |

## Milestone v0.20.0

Fait par l'orchestrateur le 2026-11-18, sur décision de Guy. **Y restent seize issues** : #1242, #1245, #1173, #1175,
#1176, #1177, #975, #984, #985, #986, #1233, #1243, #1226, #1234, #1253, #1167 (contrôlé ce jour : seize ouvertes).
**En sortent** : #1236 (état « envoyée » → 0.21.0, fonctionnalité principale envisagée), #1158 et #268 (note interne),
#944 (QR-facture, suite), #1125 (tests figés en 2026).

## Décompte au kickoff et objectif

Recompté le 2026-11-18 par les commandes du `CLAUDE.md` :

| | Tag v0.19.0 | Kickoff (2026-11-18) | Objectif au tag v0.20.0 (non barrière) |
|---|---|---|---|
| Issues ouvertes | 496 | **496** | moins de 496 |
| Bugs (`bug` ou `known-failure`) | 247 | **247** | moins de 247 |
| P1 ouvertes | 14 | **14** (dont 11 `bug`/`known-failure`) | — |

Hors les **56 issues masquées** du compte de l'agent de Beat (#1266–#1271, #1277–#1326), non comptées ; décision de Guy :
attendre leur rétablissement jusqu'au 2026-11-25, sinon recréation depuis le compte de Beat, et correction du décompte au
tag par une annotation datée.

La release vise **treize bugs** du milestone (#1245, #1173, #1175, #1176, #1177, #975, #984, #985, #986, #1233, #1243,
#1226, #1234) et jusqu'à **trois** de plus par la story de solde (#1261, #1262, #1263, si retenues). Aucune des
quatorze P1 ouvertes (dont les onze de la TVA en report assumé) n'est dans le périmètre : celles qui restent se nomment à
la rétrospective avec leur raison (`CLAUDE.md` § *Priorités des défauts*).

## Fichiers communs

| Fichier | Qui | Règle |
|---|---|---|
| `frontend/src/lib/components/invoices/InvoiceForm.svelte` | 48-13 (#1233) ; 49-1, 49-2, 49-3 | point ouvert pour Guy (arbitrage de deux épiques sur le même module) |
| `crates/kesh-qrbill/src/pdf.rs`, `types.rs` | 48-13 (#1234) ; 49-1, 49-3 ; #1327 (`largeurs.rs`, `lib.rs`) | zones distinctes ; le second arrivé résout |
| `crates/kesh-report/src/pdf.rs` | 48-11 ; #1327 | 48-11 après #1327 |
| `crates/kesh-db/src/repositories/supplier_invoices.rs`, `kesh-api/src/routes/supplier_invoices.rs` | 48-1 ; 48-12 (si clé d'idempotence) | même épique ; le second arrivé résout |
| `routes/(app)/settings/+page.svelte` | 48-13 (#1243) ; v0200-menu-sous-groupes (peut-être) | blocs distincts |
| catalogues i18n (4 locales), `sprint-status.yaml` | tous | chaque épique dans son bloc ; conflits résolus en gardant les deux côtés |
| `sitesTotal`, totaux de l'audit des migrations, schéma de test | 48-*, 49-* | recomptés après chaque fusion |
| manuels `.tex` (4 langues) | tous ; le menu change les chemins | le menu fusionne en dernier ; PDF régénérés à la release seulement |
| `CHANGELOG.md` | — | assemblé à l'intégration depuis les *Notes de version* des fiches |
| `Cargo.lock` | #1327 ; toute story qui change une dépendance | régénéré après fusion |

## Calendrier

| Jour | Quoi |
|---|---|
| mer. 2026-11-18 | kickoff (ce document) ; fiches 48-13, 48-11, 49-1 à écrire en premier |
| jeu. 19 → mer. 25 | développement sur les deux voies ; chaque story close se fusionne dans sa branche d'épique et se pousse |
| mer. 2026-11-25 | échéance des issues masquées de Beat (décision de Guy) |
| **jeu. 2026-11-26 au soir** | **coupure** : n'entre que ce qui est clos ; sortie dans l'ordre ci-dessus |
| **ven. 2026-11-27** | intégration (fusions, gates, recette de montée depuis l'image 0.19.0 sur base peuplée, revue du code d'intégration, PR, CI) ; accord de fusion de Guy vers midi ; tag et image ; NAS par Guy, **après sauvegarde complète** |

## Points ouverts pour Guy

1. **`InvoiceForm.svelte` et `kesh-qrbill` partagés entre l'Epic 48 (48-13) et l'Epic 49** : le `CLAUDE.md` veut que deux
   épiques sur le même module s'arbitrent avec le Project Lead. Recommandation : la 48-13 passe tôt ; l'Epic 49 la reprend
   à l'intégration et résout dans sa branche.
2. **Splitting préventif de la 49-1** : elle touchera vraisemblablement plus de cinq modules ; la règle commande de la
   découper avant `bmad-create-story` (par exemple 49-1a modèle et API, 49-1b écran et PDF). Ce découpage garde vos trois
   stories, mais fait passer l'Epic 49 à quatre clés ; à défaut, une section `Dérogation règle de splitting` à la fiche.
   Même examen pour 48-12 et 48-13, qui réunissent chacune quatre issues.
3. **Migration non breaking** : les lignes non chiffrées (commentaire, titre, sous-total) n'ont ni quantité ni prix. Les
   rendre nullables (`MODIFY COLUMN`) rendrait la migration **breaking** (bump de `kesh_version_min_required` et des
   crates à 0.20.0). Recommandation : garder les colonnes `NOT NULL` avec des valeurs neutres, et relâcher les `CHECK` par
   type ; la fiche tranche, la revue confirme.
4. **Sens du sous-total** (49-3) : #1242 propose « total de la section, hors total et écritures » ; restent HT ou TTC, et
   la reprise des sous-totaux dans les avoirs.
5. **Story de solde** : les quatre issues retenues (proposition : #1263, #1262, #1261, #1255) ne sont pas au milestone
   v0.20.0 ; les y rattacher est votre décision.
6. **Milestone 0.21.0** : #1236 est donnée « → 0.21.0 » mais porte aujourd'hui le milestone « Plus tard » ; il n'existe pas
   encore de milestone v0.21.0.
7. **Charge de la voie B** : sept stories, dont deux réunissant quatre issues, plus le menu, en huit jours ouvrés, en
   parallèle d'une épique à migration : la coupure tranchera, mais l'ordre de sortie (48-14, 48-2, …) suppose que les
   fiches 48-1 et 48-10, déjà écrites, se revalident vite contre la v0.19.0.
8. **#1234** (`billing_information` jamais imprimé) est **latent** tant que Kesh ne renseigne pas ce champ : le corriger
   dans la 48-13 imprime un champ toujours vide. Il reste dans la story, comme décidé ; la fiche dira s'il se limite à un
   rendu testé sur données d'essai.
