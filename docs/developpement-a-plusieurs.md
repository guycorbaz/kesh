# Développer Kesh à plusieurs — principes

Cette note explique **comment plusieurs développeurs travaillent sur Kesh en même temps** : comment le travail se
planifie, comment les versions se numérotent, comment les branches s'organisent et comment les fusions se règlent. Elle
s'adresse à un contributeur et complète `docs/prise-en-main-developpeur.md`. Les règles elles-mêmes sont au `CLAUDE.md`,
§ *Travailler à plusieurs développeurs*, qui fait foi ; cette note en donne les motifs et la marche à suivre.

*(Principes décidés par le Project Lead le 2026-11-15, quand Kesh est passé d'un développeur à plusieurs.)*

## 1. Trois niveaux, trois lieux

| Niveau | Question | Où il vit |
|---|---|---|
| **Release** | qu'est-ce qui sort, et quand ? | milestone GitHub (`v0.18.0`, `v0.19.0`, « Plus tard ») |
| **Epic** | quel chantier, porté par qui ? | champ *Epic* du projet GitHub « Kesh », et le fichier d'epic du dépôt |
| **Issue** | quel défaut ou quelle demande, traité par qui ? | GitHub Issues, avec son assigné, son epic et sa release |
| **Story** | comment l'issue est traitée, pas à pas | fiche de story et `sprint-status.yaml`, dans le dépôt |

Le **projet GitHub** est la vue d'ensemble partagée : il montre, pour chaque issue retenue, son epic, sa release et la
personne qui s'en occupe, avec une feuille de route par release, une table par epic et un tableau par développeur.
**Il ne recopie pas les stories** : un même état tenu à deux endroits finit faux à l'un des deux. Le projet dit *quoi,
pour quand, par qui* ; le dépôt dit *comment*.

**Deux réserves.** Une planification lointaine se périme : seules les deux ou trois prochaines releases sont datées, le
reste attend au milestone « Plus tard ». Et toutes les issues ouvertes ne sont pas planifiées : n'entrent au projet que
celles qu'un epic ou une release retient.

**Qui décide.** Un contributeur propose ; le **Project Lead** rattache une issue à une release, réserve un numéro d'epic
et fixe l'ordre des releases.

## 2. Les numéros d'epic se réservent

Chaque développeur travaille dans des epics à son numéro, réservés d'avance pour éviter les collisions : à ce jour 37, 38
et 39 pour un contributeur, 40 pour un autre (récupération des paiements des ventes réglées par Stripe), 36 et 41 et
suivants pour le Project Lead. Réserver un epic, c'est ajouter sa valeur au champ *Epic* du projet, avec son responsable
et son objet ; son fichier et ses stories naissent à son kickoff.

**Dans les échanges, on désigne le travail par son epic, jamais par son numéro de release** : « l'Epic 40 » reste vrai de
bout en bout, alors que le numéro de release ne devient définitif qu'au tag (§ 4).

## 3. Les versions se numérotent sur deux niveaux

- **Release planifiée** → chiffre **mineur** : 0.18.0, 0.19.0, 0.20.0. Elle a son milestone d'avance et reçoit des
  tranches d'epic.
- **Correctif urgent** → chiffre de **correction** de la dernière release publiée : 0.18.1, 0.18.2. Son milestone naît
  avec le bug.

*Pourquoi* : un correctif ne prend jamais un numéro déjà promis à une release planifiée, et le numéro dit ce qu'est la
release — « 0.18.1 » ne porte que des corrections et se pose sur la 0.18.0 sans surprise.

**Un correctif** part du **tag** de la dernière release publiée (branche `hotfix/0.18.1`), et non de `main`, qui porte
déjà du travail inachevé de la release suivante ; une fois publié, il est **reporté dans `main`**. Il ne porte **aucune
migration** : une correction qui en exige une n'est plus un correctif, elle va à la prochaine release planifiée.

*(La v0.17.8 est la dernière release numérotée à l'ancienne, où chaque release montait le chiffre de correction.)*

## 4. Qui finit le premier sort le premier

Deux développeurs préparent chacun une release : l'un la 0.19.0, l'autre la 0.20.0, et le second termine d'abord.

1. Son contenu **prend le numéro 0.19.0** : on échange les deux milestones. Un numéro n'est acquis qu'au tag ; avant, ce
   n'est qu'une intention.
2. Il fusionne dans `main`, et la release se publie depuis `main`.
3. L'autre **reprend `main` dans sa branche**, résout les conflits, puis rejoue **tous** les gates et l'E2E complet :
   le second arrivé paie l'intégration. Si les deux releases portent des migrations, leur ordre se contrôle à ce
   moment, sur une base peuplée par la release qui vient de sortir ; la recette de montée part toujours de la dernière
   image publiée.

Attendre l'autre ferait vieillir la branche et grossir les conflits. Une release commune reste possible quand l'autre
est tout proche du but — le Project Lead en décide au cas par cas.

**Un correctif dépassé** : si une release planifiée sort pendant qu'un correctif se prépare depuis la précédente, le
correctif se publie depuis la nouvelle (0.19.1 plutôt que 0.18.2) — il n'y a qu'une installation en production —, sauf
si le Project Lead veut corriger sans installer encore le reste.

**Seul le Project Lead pose un tag et publie une image** (ou l'orchestrateur sur son mandat explicite) : un seul point
ordonne les publications, et deux tags ne peuvent pas se croiser.

## 5. Les branches

| Branche ou tag | Rôle |
|---|---|
| `epic/N-slug` | une **tranche** d'epic : ce que l'epic livre dans une release, par son développeur |
| `story/N-M-slug` | une story, partie de sa tranche et fusionnée dans sa tranche |
| `main` | l'intégration : ce qui est terminé, revu, et attend sa release |
| tag `vX.Y.Z` | la production : le code exact de l'image publiée |
| `hotfix/X.Y.Z` | un correctif urgent, parti du tag, reporté ensuite dans `main` |

Le cycle d'une tranche :

1. Créer `epic/40-stripe` depuis `main` à jour.
2. Pour chaque story : `story/40-1-…` depuis la tranche, développée, revue, **gates locaux verts**, puis PR vers la
   tranche — la CI y tourne comme vers `main` (§ 7).
3. Après chaque release publiée par un autre, **reprendre `main` dans la tranche**.
4. La tranche terminée : PR de `epic/40-stripe` vers `main`, **revue du code d'intégration** (§ 6), fusion.
5. **La tranche se clôt** ; la suivante repart de `main`.

**Pourquoi une branche par tranche, et non une par epic entier.** Le dépôt fusionne en *squash* : toute la branche
devient un seul commit dans `main`. Une branche qui continuerait après sa fusion ne reconnaîtrait pas son propre travail
dans `main`, et chaque reprise de `main` rejouerait de faux conflits. Un epic qui livre en plusieurs releases a donc une
branche par release.

**Pas de branche `develop`** : `main` ne reçoit que des tranches terminées, et le tag tient lieu de branche de production.
Une branche de plus ajouterait une fusion à chaque release sans rien gagner en sûreté.

**On ne touche ni aux branches ni aux bases de test d'un autre.**

## 6. Les fusions

### Prévenir

La plupart des conflits de Kesh sont prévisibles : ils tombent dans les fichiers que tout le monde touche, non dans le
code métier. Chacun a sa règle :

| Fichier | Règle |
|---|---|
| `CHANGELOG.md` | personne ne l'écrit en cours d'epic ; chaque story porte sa *Note de version*, assemblée à la release |
| PDF des manuels | jamais commités dans une branche d'epic ou de story — un conflit sur un PDF ne se résout pas ; seul le `.tex` change |
| catalogues i18n (quatre locales), `sprint-status.yaml` | chaque epic écrit dans son propre bloc ; le conflit se résout en gardant les deux côtés |
| compteurs (`sitesTotal` de `i18n-keys.test.ts`, totaux de l'audit des migrations, schéma de test) | se **recomptent** depuis la source après la fusion |
| migrations | un fichier chacune, nommé par horodatage ; l'ordre se contrôle à l'intégration |
| `Cargo.lock` | se régénère après la fusion |

Et deux gestes : **au kickoff**, chaque fichier d'epic déclare les modules qu'il touchera — deux epics sur le même
module se voient avant la première ligne, et l'on choisit l'ordre ou le partage ; **en cours de route**, chacun reprend
`main` souvent, pour régler dix petits conflits frais plutôt qu'un gros conflit vieux de deux mois.

### Résoudre

- **Le second arrivé résout, dans sa branche**, jamais dans `main`.
- La résolution **garde les deux intentions** : la règle, le test et le texte de chacun. Si elles se contredisent, on
  ne tranche pas seul : on en parle à l'autre, et le Project Lead arbitre.
- Après la résolution, **tous les gates et l'E2E complet**, puisque la branche a changé.

### Les conflits que Git ne voit pas

Le plus dangereux : deux epics qui changent **la même règle dans deux fichiers différents** — l'un modifie la lecture
d'un montant, l'autre ajoute un écran qui lit les montants à l'ancienne. Git fusionne sans rien dire, et le résultat est
faux. Deux filets : les tests, rejoués après chaque fusion ; et la **revue du code d'intégration**, obligatoire à chaque
PR d'une tranche vers `main`, qui relit les commits de fusion et cherche ces interactions.

## 7. La CI

- Elle tourne sur les PR vers `main` **et vers les branches `epic/…`** : une story passe la même CI avant d'entrer dans
  sa tranche. Sur une tranche, elle informe sans bloquer ; vers `main`, ses quatre jobs sont requis.
- Elle ne remplace pas les gates locaux, qui restent la règle avant chaque `push` (`CLAUDE.md` § *Test Locally First*).
- Le cache Rust n'est écrit que depuis `main` : le cache du dépôt est plafonné à 10 Go, et chaque branche qui écrirait
  le sien ferait tout recompiler aux autres. Détail : `docs/ci.md`.
