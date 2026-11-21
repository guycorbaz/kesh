# Guide du développeur — travailler sur Kesh

Ce guide s'adresse à un développeur qui rejoint Kesh. Il dit **ce qu'il faut savoir avant d'écrire la première ligne**,
comment monter le poste et vérifier son travail, comment plusieurs développeurs travaillent en même temps, et comment les
versions se numérotent et se publient. Il renvoie aux documents du dépôt plutôt que de les recopier.

**Le `CLAUDE.md` (racine) est la référence des règles** : ce guide les explique, en donne les motifs et la marche à
suivre, et y renvoie. En cas d'écart, c'est le `CLAUDE.md` qui fait foi.

Il se compose de trois parties :

- **I. Prise en main** : ce qu'il faut lire, installer, compiler et tester ;
- **II. Règles de travail** : proposer un changement, les branches, les epics, les fusions, la revue, la reprise après un
  crash ;
- **III. Versions** : la numérotation, les releases, l'API.

*(Ce guide remplace les deux notes « Prise en main » et « Développer à plusieurs », fusionnées le 2026-11-21 ; les
principes de travail à plusieurs ont été décidés par le Project Lead le 2026-11-15, et révisés le 2026-11-21.)*

---

# I. Prise en main

## 1. Avant tout : Kesh tient des livres réels

Kesh est **en production** : le Project Lead y tient sa comptabilité réelle depuis le 2026-10-31. Trois règles en
découlent, sans exception :

- **Aucune donnée réelle** dans le dépôt, les issues, les commits, les captures ou les fixtures : ni IBAN, ni nom, ni
  montant, ni contenu d'une pièce. Le dépôt est **public**. Les tests emploient des données synthétiques.
- **Jamais l'installation de production** : aucun essai, aucune recette, aucune clé d'essai ne se joue sur l'instance du
  Project Lead. Tout se joue sur une installation jetable (la vôtre, § 3).
- **Une migration n'est jamais anodine** : elle s'appliquera à des livres qu'on ne refait pas, et toute migration rend le
  retour à la version précédente dépendant de la sauvegarde d'avant la mise à jour. Avant d'en écrire une, parlez-en
  (§ 13).

## 2. Ce qu'il faut lire, dans cet ordre

1. **`CLAUDE.md`** (racine) — les règles du dépôt : tests, revues, migrations, priorités des défauts, commits. Il est
   long ; lisez au moins *Test Locally First*, *Review Iteration Rule*, *Priorités des défauts*, *Issue Tracking Rule*,
   *Migration breaking policy* et *Commits, branches et push*.
2. **`README.md`** — l'aperçu, la structure du dépôt, le démarrage rapide.
3. **`docs/testing.md`** — les patterns de tests, la base de dev jetable, la recette E2E et **la liste des échecs E2E
   attendus** (§ *Les échecs attendus*).
4. **`docs/ci.md`** — ce que la CI exécute, et ce qu'elle n'exécute pas.
5. Selon votre sujet : `docs/MULTI-TENANT-SCOPING-PATTERNS.md` (toute requête est scopée par société),
   `docs/optimistic-locking-patterns.md`, `docs/i18n-key-ownership-pattern.md`, `docs/api-external.md`, le PRD et
   l'architecture sous `_bmad-output/planning-artifacts/`.

## 3. Le poste de dev

**Prérequis** : Rust (la version est fixée par `rust-toolchain.toml`, installée d'office par rustup), Node.js ≥ 20, Docker
et Docker Compose. `cargo install cargo-nextest` est recommandé.

```sh
git clone https://github.com/guycorbaz/kesh.git && cd kesh
bash scripts/install-hooks.sh        # refuse les commits directs sur main
docker compose -f docker-compose.dev.yml up -d mariadb
```

- **La base de dev est en RAM** (tmpfs) : rien ne survit à un redémarrage du conteneur, et c'est voulu. Le conteneur se
  crée **depuis le clone principal, jamais depuis un worktree** (son montage des droits est relatif au répertoire de
  lancement). Détail et remise à zéro : `docs/testing.md` § *Base de dev jetable*.
- **Migrations et données de dev** : `sqlx migrate run --source crates/kesh-db/migrations`, puis
  `docker exec -i kesh-mariadb-dev mariadb -uroot -pkesh_dev_root kesh < scripts/seed-dev-db.sql`.
- **Lancer l'application** : `README.md` § *Démarrage rapide* (backend `cargo run -p kesh-api`, frontend
  `cd frontend && npm install && npm run dev`). Au premier accès, l'écran `/setup` crée l'administrateur.
- **Avant chaque gate complet, remettez la base à zéro**, même après un run vert : un test peut laisser des lignes qui
  font rougir le suivant (`CLAUDE.md` § *Base de gate*).

## 4. Vérifier son travail en local

La règle est simple : **avant chaque `git push`, la même série de contrôles que la CI**.

**Backend** (racine du dépôt) :

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
scripts/test-fast.sh --ci        # fmt + clippy + nextest, MariaDB de dev démarré
```

La CI lance `cargo test --workspace -j1 -- --test-threads=1` (un processus par binaire) ; `scripts/gate-ci.sh <base>` la
rejoue en local quand un doute existe (`docs/ci.md`).

**Frontend** (`frontend/`) :

```sh
npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build
```

**E2E** (Playwright) : la recette exacte est dans `docs/testing.md` § *Prérequis Playwright local* — recopiez-la, ne la
reconstituez pas (`KESH_COOKIE_SECURE=false` y est indispensable en HTTP local). **La suite ne passe pas au vert** : sept
échecs connus (KF-029, #97) sont attendus. Un rouge se juge fichier par fichier contre la liste de `docs/testing.md`, jamais
au nombre.

**Mémoire** : un gate complet est gourmand. `scripts/mem-guard.sh <commande>` l'exécute sous un plafond de mémoire (sans
effet si systemd utilisateur est absent) ; `scripts/test-fast.sh` le fait seul.

---

# II. Règles de travail

## 5. Proposer un changement

- **Une issue d'abord**, sur GitHub, avec le gabarit qui convient (`.github/ISSUE_TEMPLATE/` : bug, known failure,
  demande). GitHub Issues est l'unique registre : pas de liste de défauts dans le dépôt. **Un point qu'on décide de ne pas
  traiter s'ouvre aussi en issue** (« on trace toujours »), une issue par problème.
- **Une priorité par défaut**, dès la création (`CLAUDE.md` § *Priorités des défauts*) : **P1** ce qui fausse la
  comptabilité (écriture, solde, compte ou total faux), perd ou corrompt des données, ou compromet la sécurité ; P2 un
  comportement faux sans effet sur les comptes, ou un manquement de conformité légale qui ne produit pas d'erreur dans les
  livres ; P3 un manuel qui décrit autre chose que le code, un test défaillant, de la dette ; P4 l'orthographe, la
  traduction, le cosmétique. **Un P1 passe en priorité dans la release suivante**, quitte à une release dédiée.
- **Une branche par travail** : un epic `epic/…` depuis `main` à jour, ses stories `story/…` depuis la branche de l'epic ;
  hors epic, `fix/…`, `chore/…`, `docs/…` depuis `main` (§ 7).
- **Commits** : un par étape, au format Conventional Commits (`fix(kesh-api) : …`), chacun en `refs #N`. ⛔ **Aucun mot-clé
  de fermeture** (`closes`, `fixes`, `resolves`) dans un message de commit : il ne s'écrit que dans le corps de la **PR**,
  que le dépôt fusionne en squash.
- **PR** : gabarit `.github/PULL_REQUEST_TEMPLATE.md` ; la CI exécute quatre jobs (backend, frontend, image Docker, smoke
  E2E), tous requis pour la fusion. Une PR qui ne touche que `_bmad-output/` les saute.
- **Documentation dans la même PR que le code** : un comportement visible qui change met à jour le manuel concerné
  (`docs/manual/fr/*.tex`, PDF régénéré à la release), les **quatre locales** (`crates/kesh-i18n/locales/*/messages.ftl`),
  et la note de version au CHANGELOG à la release.
- **Qualité** : DRY, code documenté (`///` en Rust), tout ce qui se teste est testé ; un test neuf doit **rougir sans le
  correctif** (le montrer). Une fonctionnalité visible s'accompagne de captures d'écran.

## 6. Trois niveaux, trois lieux

| Niveau | Question | Où il vit |
|---|---|---|
| **Release** | qu'est-ce qui sort, et quand ? | milestone GitHub, nommé par date (« Vendredi 2026-11-27 », « Plus tard ») |
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

**Les numéros d'epic se réservent.** Chaque développeur travaille dans des epics à son numéro, réservés d'avance pour
éviter les collisions. Réserver un epic, c'est ajouter sa valeur au champ *Epic* du projet, avec son responsable et son
objet ; son fichier et ses stories naissent à son kickoff. Le prochain numéro d'epic interne neuf est le **55**. Le
contenu de vos epics se décide avec le Project Lead.

**Dans les échanges, on désigne le travail par son epic et par la date de sa release, jamais par un numéro de version** :
« l'epic des paiements » ou « la release du vendredi 27 » restent vrais de bout en bout, alors que le numéro de version ne
devient définitif qu'au tag (§ 14).

## 7. Les branches

| Branche ou tag | Rôle |
|---|---|
| `epic/N-slug` | un epic, par son développeur |
| `story/N-M-slug` | une story, partie de la branche de son epic et fusionnée dans celle-ci |
| `main` | l'intégration : ce qui est terminé, revu, et attend sa release |
| tag `vX.Y.Z` | la production : le code exact de l'image publiée |
| `hotfix/X.Y.Z` | un correctif parti du tag, reporté ensuite dans `main` (§ 15) |

Le cycle d'un epic :

1. Créer `epic/40-paiements` depuis `main` à jour.
2. Pour chaque story : `story/40-1-…` depuis la branche de l'epic, développée, revue, **gates locaux verts**, puis PR
   vers `epic/40-paiements` — la CI y tourne comme vers `main` (§ 10).
3. Après chaque release publiée par un autre, **reprendre `main` dans la branche de l'epic**.
4. L'epic terminé : PR de `epic/40-paiements` vers `main`, **revue du code d'intégration** (§ 9), fusion, puis la release.
5. **La branche se clôt** ; l'epic suivant repart de `main`.

**Pourquoi une branche ne se fusionne qu'une fois.** Le dépôt fusionne en *squash* : toute la branche devient un seul
commit dans `main`, et la protection de `main` impose un historique linéaire. Une branche qui continuerait après sa
fusion ne reconnaîtrait pas son propre travail dans `main`, et chaque reprise de `main` rejouerait de faux conflits.
C'est pourquoi un epic tient dans une release (§ 8). Si, par exception, une branche d'epic continue après une entrée
dans `main`, elle reprend `main` aussitôt.

**Pas de branche `develop`** : `main` ne reçoit que des epics terminés, et le tag tient lieu de branche de production.
Une branche de plus ajouterait une fusion à chaque release sans rien gagner en sûreté.

**Une branche d'epic n'est pas une base** : on part de `main` ; bâtir sur une branche d'epic non publiée demande l'accord
du Project Lead.

**On ne touche ni aux branches ni aux bases de test d'un autre.**

## 8. Les epics : petits, grands, dépendants

- **Un epic tient dans une release** : au plus **six stories**, story de solde comprise, fonctionnalité nouvelle
  comprise. Un epic qu'on ne voit pas finir dans la semaine se **découpe au kickoff** en deux ou trois epics. Il entre
  dans `main` **une seule fois, terminé**. Une release peut en revanche réunir plusieurs epics.
  *Pourquoi* : un petit epic se planifie, se livre et se juge d'un seul tenant ; il se termine vite, sa branche vieillit
  peu et ses conflits avec le travail des autres restent petits.
- **Pas de story trop longue ni trop complexe** : une story qui touche plus de cinq modules se découpe dès sa fiche
  (`CLAUDE.md` § *Règle de splitting préventif*).
- **Grand epic par dérogation** : un chantier qui ne se découpe vraiment pas peut rester dans sa branche jusqu'à sa fin,
  sur décision du Project Lead prise au kickoff. Le fichier d'epic porte alors une section « Dérogation » (raison, nombre
  de releases estimé). Les releases partent sans lui ; il **reprend `main` après chaque release et rejoue ses gates** ;
  ses issues restent au milestone « Plus tard » jusqu'à la release qui l'accueille ; la revue du code d'intégration se fait
  à l'entrée. Sa migration se redate à l'intégration, après la dernière publiée (à vérifier au code avant d'appliquer).
- **Dépendances** : un epic qui dépend d'un autre, ou qui en bloque un, le déclare **au kickoff** (« dépend de »,
  « bloque » dans le fichier d'epic, et relations d'issues sur GitHub). L'ordre d'intégration les suit.
- **Un epic qui n'aboutit pas à la coupure se réduit** : on ne retarde pas la release, on réduit son étendue. Les stories
  non closes sortent vers un epic ultérieur (de six stories au plus) ; vous réduisez le vôtre vous-même, ou il attend la
  semaine suivante sans retenir les autres. L'epic réduit se clôt avec la release.

## 9. Les fusions

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

Et deux gestes : **au kickoff**, chaque fichier d'epic déclare les fichiers et modules qu'il touchera — deux epics sur le
même module se voient avant la première ligne, et l'on choisit l'ordre ou le partage ; **en cours de route**, chacun
reprend `main` souvent, pour régler dix petits conflits frais plutôt qu'un gros conflit vieux de deux mois.

### Résoudre

- **Le second arrivé résout, dans sa branche**, jamais dans `main`.
- La résolution **garde les deux intentions** : la règle, le test et le texte de chacun. Si elles se contredisent, on
  ne tranche pas seul : on en parle à l'autre, et le Project Lead arbitre.
- Après la résolution, **tous les gates et l'E2E complet**, puisque la branche a changé.

### Les conflits que Git ne voit pas

Le plus dangereux : deux epics qui changent **la même règle dans deux fichiers différents** — l'un modifie la lecture
d'un montant, l'autre ajoute un écran qui lit les montants à l'ancienne. Git fusionne sans rien dire, et le résultat est
faux. Deux filets : les tests, rejoués après chaque fusion ; et la **revue du code d'intégration**, obligatoire à chaque
PR d'un epic vers `main`, qui relit les commits de fusion et cherche ces interactions.

## 10. La CI

- Elle tourne sur les PR vers `main` **et vers les branches `epic/…`** : une story passe la même CI avant d'entrer dans
  la branche de son epic. Sur une branche d'epic, elle informe sans bloquer ; vers `main`, ses quatre jobs sont requis.
- **Aucun allègement n'est prévu pour les PR vers `epic/…`** (décision du Project Lead du 2026-11-21) ; la question se
  rouvre si les files d'attente de la CI gênent.
- Elle ne remplace pas les gates locaux, qui restent la règle avant chaque `push` (`CLAUDE.md` § *Test Locally First*).
- Le cache Rust n'est écrit que depuis `main` : le cache du dépôt est plafonné à 10 Go, et chaque branche qui écrirait
  le sien ferait tout recompiler aux autres. Détail : `docs/ci.md`.

## 11. La revue

Les changements de Kesh passent par une boucle de revue adversariale (`CLAUDE.md` § *Review Iteration Rule*) : elle se
relance tant qu'une passe trouve un défaut au-dessus de LOW. Le Project Lead fixe avec vous le niveau de revue attendu
pour vos PR.

**La passe ciblée et la définition d'un « module ».** Quand la boucle converge, la passe suivante peut être une passe
ciblée, sur le seul commit de la dernière remédiation. Elle ne suffit pas si le patch touche plusieurs modules ou change
une règle métier. **Un module est un crate, ou un domaine du frontend (`frontend/src/lib/features/<domaine>`), dont le
code de production change.** Un fichier de test qui ne teste que le correctif, la fiche de story et le manuel ne comptent
pas : un correctif du frontend accompagné de son test Rust reste un patch d'un seul module. Un patch qui change une règle
métier reprend toujours le protocole complet.

## 12. Reprise après un crash

Quand la machine de travail tombe (coupure, mémoire saturée, session perdue), les agents et les processus qu'elle
portait meurent sans prévenir. Trois règles pour ne pas perdre le travail :

1. **Un agent commite à chaque étape franchie** (test rouge, correctif vert, remédiation), pas seulement quand il fait une
   pause, et **pousse sa seule branche `story/…`** à chaque étape : le travail survit ainsi à la perte de la machine.
   Jamais une branche d'epic, `main` ni un tag.
2. **Quand un agent rend la main avec un travail de fond en cours** (des gates qui tournent, une attente), l'orchestrateur
   arme lui-même une attente sur sa fin : l'agent ne reprendra pas de lui-même.
3. **Après un crash, on suit la procédure écrite** dans `docs/agents-orchestration.md` § *Reprise après un crash* : lister
   les worktrees et leur état, chercher les processus orphelins et les verrous tenus, contrôler qu'aucune mutation n'est
   restée, et seulement alors commiter et relancer.

Si vous travaillez seul, les mêmes réflexes valent : commits fréquents, branche poussée, et vérification d'une mutation de
test oubliée avant de commiter quoi que ce soit.

## 13. Coordination

- **D'autres travaux avancent en parallèle.** Deux points se règlent au début de chaque epic : les **fichiers communs**
  (qui touche quoi, et dans quel ordre les fusionner) et les **migrations**, numérotées par horodatage et dont l'ordre se
  contrôle à l'intégration.
- **Migrations** : toute PR qui en ajoute une suit la politique P1 à P8 du `CLAUDE.md` (bump de version minimale pour une
  migration *breaking*, ligne à `docs/migrations-idempotence-audit.md`, triage des backfills, une migration appliquée ne se
  modifie plus jamais). **Annoncez-la avant de l'écrire** : une migration s'annonce d'avance au plan du projet GitHub, car
  elle oblige l'administrateur à une sauvegarde complète avant la mise à jour. Une story sortie de son epic emporte sa
  migration, et la release qui la portera se réannonce.
- **Une question, un désaccord, un défaut trouvé** : une issue, ou le Project Lead directement.

---

# III. Versions

## 14. La numérotation `a.b.c`

Décision du Project Lead du 2026-11-21 (`CLAUDE.md` § *Numérotation des versions*) :

- **`a`** vaut 0 tant que Kesh ne tient pas complètement les comptes d'un utilisateur ordinaire — le 0 parle des
  fonctions, non de l'usage — et passe à 1 le jour où il les tient. Le critère de la 1.0 reste à écrire (candidat : la
  première clôture d'exercice tenue dans Kesh, complétée d'une liste de fonctions du PRD). Le sens de `a` = 2 reste à
  définir.
- **`b` monte quand la release porte une migration ou une rupture du contrat de l'API, et seulement alors.**
- **`c` monte pour toute autre release**, celle du vendredi comprise. **Une release `c` ne porte jamais de migration.**
- **Le numéro ne suit plus le calendrier.** Les milestones se nomment par date (« Vendredi 2026-11-27 », « Plus tard ») ;
  le numéro de version s'attribue au tag. Avec les autres développeurs, on parle de dates.

*Ce que cela signifie pour l'administrateur d'une installation* : si `c` monte, il remplace l'image, et revient en
arrière en remettant l'image précédente ; si `b` monte, il lit les notes de version, et, s'il y a une migration, une
sauvegarde complète (base, `.keshbackup`, dossier des documents) est obligatoire avant la mise à jour, le retour passant
par cette sauvegarde (manuel d'administration, « Lire le numéro de version »).

## 15. La cadence des releases

- **Une release planifiée chaque vendredi**, pendant les semaines de présence du Project Lead. La date tient ; c'est le
  périmètre qui s'ajuste.
- **Des releases en cours de semaine** quand elles apportent quelque chose qui compte aux utilisateurs : un P1, un bug
  d'usage, un epic terminé. Elles ne partent pas d'office : chacune coûte les gates complets, la recette de montée et
  l'audit de la documentation.
- **Toutes les releases partent de `main`**, qui ne porte que du fini. Une branche `hotfix/X.Y.Z`, partie du tag, ne
  sert que si `main` porte ce qu'on ne veut pas livrer (par exemple une migration qui attend le vendredi). Le correctif est
  ensuite reporté dans `main`. Des exceptions restent possibles, sur décision du Project Lead.
- **Les migrations sortent de préférence le vendredi**, sans s'interdire le milieu de semaine.

| Moment | Ce qui se passe |
|---|---|
| du vendredi au jeudi | les stories avancent ; chacune se fusionne dans la branche de son epic dès sa clôture |
| **jeudi soir : coupure** | n'entre que ce qui est clos (revue terminée, gates et E2E complet verts) |
| vendredi matin | intégration : dernières fusions, gates, recette de montée, revue du code d'intégration, PR |
| vendredi midi | accord de fusion du Project Lead, puis tag et image |
| vendredi après-midi ou samedi | le Project Lead met à jour l'installation de production |

- **Chaque release planifiée corrige au moins un ou deux bugs**, en plus de ses epics, choisis par impact : un P1 d'abord,
  sinon ce qui touche l'usage réel, sinon le plus ancien. Le dosage s'adapte : certaines semaines portent surtout des
  corrections et seulement une petite fonctionnalité.
- **Une release dédiée à un P1** part sans attendre le vendredi, et ne remplace pas la release de la semaine.
- **Une semaine d'absence du Project Lead est une semaine sans release** : il annonce ses dates d'avance, et le travail
  continue pour la semaine de son retour.

## 16. Qui finit le premier sort le premier

Deux développeurs préparent chacun une release, et le second termine d'abord.

1. Son contenu sort le premier : on échange les dates et les milestones. Un numéro de version n'est acquis qu'au tag ;
   avant, ce n'est qu'une intention.
2. Il fusionne dans `main`, et la release se publie depuis `main`.
3. L'autre **reprend `main` dans sa branche**, résout les conflits, puis rejoue **tous** les gates et l'E2E complet : le
   second arrivé paie l'intégration. Si les deux releases portent des migrations, leur ordre se contrôle à ce moment, sur
   une base peuplée par la release qui vient de sortir ; la recette de montée part toujours de la dernière image publiée
   (`scripts/recette-montee.sh`).

Attendre l'autre ferait vieillir la branche et grossir les conflits. Une release commune reste possible quand l'autre
est tout proche du but — le Project Lead en décide au cas par cas.

**Seul le Project Lead pose un tag et publie une image** (ou l'orchestrateur sur son mandat explicite) : un seul point
ordonne les publications, et deux tags ne peuvent pas se croiser.

## 17. Les versions de l'API

Règle décidée par le Project Lead le 2026-11-21 (`CLAUDE.md` § *Versions de l'API*). Sa mise en œuvre est l'objet d'une
demande de changement (CR) à part : aujourd'hui, ce guide décrit la règle à venir, non le comportement du code.

- **Seul le majeur figure dans l'URL** : `/api/v1/`, `/api/v2/`. La version fine se lit dans `/health`, la description
  OpenAPI et `docs/api-external.md`.
- **Un ajout** (route, champ facultatif, code d'erreur neuf) ne change rien pour le client : `c` monte.
- **Une rupture** (renommage, suppression, type changé, champ devenu obligatoire) : nouvelle version de l'API, et `b`
  monte.
- **La correction d'un comportement contraire au contrat documenté** : annoncée ⚠️ en tête des notes de version, `b`
  monte, sans nouvelle version de l'API.
- **Deux versions sont servies**, l'actuelle et la précédente. Le retrait de la précédente est annoncé d'avance par les
  en-têtes `Deprecation` et `Sunset`.
- **Une route inchangée** garde le même handler sous les deux préfixes, testé une seule fois. **Une route modifiée** vit
  en `v2` ; la `v1` devient un adaptateur, testé sur son seul contrat.
- **Une garde statique** vérifie que chaque route `v1` est présente, adaptée ou déclarée retirée.
