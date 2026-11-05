# Rétrospective — la série 0.17 (v0.17.0 à v0.17.3)

**Date : 2026-11-05.** Tenue **en autonomie**, sans dialogue animé, sur demande de Guy (« quand la release est prête,
procède à la rétrospective puis à la revue de projet », 2026-11-05) : aucune décision du Project Lead n'est inventée
ici ; ce qu'une rétrospective animée lui aurait demandé est au § 10, chaque question avec la recommandation de l'équipe.
**À relire par Guy** ; la revue de projet suit.

**Ce que couvre la série.** Ce n'est pas un epic : aucun kickoff, aucune clé `epic-N` au registre. Ce sont quinze stories
**hors epic**, nées après la rétrospective de l'Epic 32 et livrées par quatre releases en trois jours, pendant que Guy
tenait ses livres réels dans Kesh. Le périmètre est celui des clés `v016-1-mcp-sorties` à `v0172-session` du
`sprint-status.yaml` (v016-\*, v017-\*, v0171-\*, v0172-\*), du tag `v0.16.0` (2026-10-28, 10:57 +01:00) au tag
`v0.17.3` (2026-11-05, 13:00 +01:00).

**Sources** : les dix-huit fiches du périmètre (`v016-1-mcp-sorties.md` à `v0172-session.md`), leurs Change Logs et
Dev Agent Records ; les dix registres `*-choix-autonomes.md` ; les prompts versionnés à côté des fiches ; les rapports
hors dépôt `kesh-gate-logs/v016*`, `v017*`, `v0171*`, `v0172*`, `rel016/`, `rel0161/`, `rel017/`, `rel0171/`,
`rel0172/`, `rel0173/` (lecture seule) ; `CHANGELOG.md` ; `git log` et `git diff` entre les tags ; `gh issue list`,
`gh issue view`, `gh pr view`, `gh release view` (lecture) ; `journalctl -u systemd-oomd` (lecture) ; le journal de
l'orchestrateur (`memory/epic-15-autonome.md`, entrées du 2026-10-28 au 2026-11-05, `kesh-en-production.md`,
`serveur-mcp-decisions.md`), cité comme **« relevé de l'orchestrateur »** quand aucune source du dépôt ne porte le
fait. Les scripts de décompte de cette rétrospective sont dans `kesh-gate-logs/retro017/` (`compte.py`, `mouv.py`,
`periode.py`, `diff.py`, `apres.py`, sur l'instantané `issues.json` tiré le 2026-11-05).

⛔ **Aucune donnée réelle de Guy n'est écrite ici** — ni IBAN, ni nom de tiers, ni montant, ni contenu d'une pièce, ni
solde du NAS. Le dépôt est public. Les défauts trouvés sur les livres réels sont décrits par leur structure.

---

# Partie 1 — Revue de la série

## 1. Ce qui a été livré

| Release | Date (tag) | Stories | Contenu | PR |
|---|---|---|---|---|
| *v0.16.1-rc1* | 2026-10-29 | v016-1 | correctif du serveur MCP (#811 P2 : le contenu d'un envoi validé n'atteignait pas Claude Code ; #812 P4) ; **pré-version, jamais finale** — son contenu passe dans la 0.17.0 (décision de Guy du 2026-10-30) | #815, fermée sans fusion le 2026-11-03 |
| *v0.17.0-rc1* | 2026-10-31 | v016-1, v016-2a, v016-2b, v017-1, v017-2, v017-3a, v017-3b | pré-version installée sur le NAS le 2026-10-31 ; :latest non déplacé | (tag sur `release/0.17.0`) |
| **v0.17.0** | 2026-11-03 | + v017-4a, v017-4b, v017-5, v017-6 (onze en tout) | le serveur MCP ne rend plus que du texte (#811, #812) ; **relevé bancaire saisi à la main** (#816, migration `20261030000001`) ; **#789** (quick-xml 0.42, P1 sécurité) ; l'assistant d'installation dit le motif d'un refus (#809) ; rattacher une transaction à une écriture existante (#671) ; refus d'une ligne fournisseur sous le centime (#687) ; **factures fournisseur en brouillon → validée → payée** (#849, migration `20261031000001`, avec #853, #830, #835, #831) ; QR d'une pièce scannée relu à meilleure résolution (v017-5, refs #854) et verrou d'import rendu à l'abandon de la requête (#863) ; petites corrections (#824, #832) | #846 |
| **v0.17.1** | 2026-11-04 | v0171-correctifs | corrective, sans migration : QR à lignes CR+LF (#854), quantités `1` et non `1.0000` (#866, #878), dates suisses (#871, #879) | #884 |
| **v0.17.2** | 2026-11-04 | v0172-annulees, v0172-correctifs | corrective, sans migration : une facture fournisseur annulée **repasse en brouillon** (#868) ; verrou du compte à l'import bancaire (#840) ; liste fournisseur paginée (#870) ; menu latéral (#851) ; taux de TVA désactivé (#882) ; date « à vérifier » (#885) ; dates des écrans comptables (#880) ; LOW de la 0.17.1 (#883) | #894 |
| **v0.17.3** | 2026-11-05 | v0172-session | **la session ne se ferme plus en pleine saisie** (#876) ; durée d'inactivité réglée dans l'administration, 60 min par défaut (CR #891, qui modifie FR13 du PRD) ; une migration non breaking (`20261104000001_installation_settings`) | #900 |

**Quinze stories livrées** (`done` au registre, hors `split`) ; trois fiches parentes `split` (v016-2, v017-3, v017-4) ;
une fiche `backlog` (v017-3c, avertissements sur les propositions, tracée en #828). Recompte :

```sh
python3 -c "import yaml;d=yaml.safe_load(open('sprint-status.yaml'))['development_status'];k=list(d);\
i=k.index('v016-1-mcp-sorties');j=k.index('v0172-session');import collections;\
print(collections.Counter(d[x] for x in k[i:j+1]))"        # done 15, split 3, backlog 1
```

Total du dépôt : **322 stories** `done` (307 à la rétrospective 32, plus ces quinze), par la règle de comptage du
`CLAUDE.md`.

**Une PR par release, et non par story** : quatre PR fusionnées pour le code (#846, #884, #894, #900), contre 34 à
l'Epic 32. Les stories s'intègrent sur une branche `release/0.17.x` par un agent d'intégration, puis une seule PR part.
C'est le changement de méthode le plus visible de la série (§ 7.1, § 7.2).

### 1.1 Ce que la série n'a pas livré, et qui était prévu

La rétrospective 32 (§ 9) annonçait un **Epic 33** pour la 0.17.0 : **les propositions de l'agent** et **#235** (pièces
justificatives sur les écritures, P1), conformément aux décisions de Guy du 2026-10-24 (`serveur-mcp-decisions.md`). **Ni
l'un ni l'autre n'est venu**, et aucun kickoff n'a eu lieu. La 0.17.0 a pris un autre contenu, décidé par Guy pas à pas :
le correctif MCP de la recette (#811) devient une 0.16.1, que le relevé saisi à la main (#816, « je propose rc2 ») puis la
fusion de candidates (« on intègre la rc1 actuelle et on fait toute la recette et les tests en même temps ») font
devenir la 0.17.0 ; s'y ajoutent les défauts que la tenue des livres réels a fait surgir. **La série a suivi la
production, non le plan** — ce n'est pas un écart à reprocher, c'est un fait à écrire : la 0.17 n'est pas la 0.17 des
décisions du 2026-10-24, et la feuille des décisions (`serveur-mcp-decisions.md`) n'est plus à jour sur ce point (§ 10).

### 1.2 Chiffres de produit

| | v0.16.0 | v0.17.0 | v0.17.1 | v0.17.2 | v0.17.3 |
|---|---|---|---|---|---|
| Migrations | 86 | **88** | 88 | 88 | **89** |
| Version Cargo | 0.16.0 | 0.17.0 | 0.17.1 | 0.17.2 | 0.17.3 |
| Tests backend (nextest) | 4456 (7 ignorés) | 4673 (12) | 4683 (12) | 4714 (12) | **4739** (12) |
| Tests frontend (Vitest) | 2150 (181 fichiers) | 2398 (195) | 2433 (204) | 2527 (206) | **2623** (211) |
| E2E (réussis / échecs / ignorés) | 325 / 7 / 17 | 334 / 8 / 18 | 334 / 8 / 18 | 342 / 8 / 18 | **343 / 7 / 18** |

*(Migrations : `git ls-tree <tag> crates/kesh-db/migrations/ | grep -c '\.sql'`. Version :
`git show <tag>:crates/kesh-api/Cargo.toml`. Tests : lignes « Summary » de `kesh-gate-logs/rel017/final-test-fast-f1.log`,
`rel0171/gate-backend.log`, `rel0172/gate-backend.log`, `rel0173/gate-backend.log` ; « Tests … passed » de
`rel017/final-vitest-f1.log`, `rel0171/gate-frontend.log`, `rel0172/gate-frontend.log`, `rel0173/front-test-unit.log` ;
E2E : `rel017/final-e2e-f1.log`, `rel0171/e2e.log`, `rel0172/e2e.log`, `rel0173/e2e.log`. La v0.16.0 vient de la
rétrospective 32, § 1.1. Les sept échecs sont ceux de la KF-029 (#97) ; le huitième des runs 0.17.0 à 0.17.2 est une
pollution d'état rejouée verte seule — KF-052 ou `vat-rates.spec.ts:55` selon les Change Logs.)*

**Diff** : `git diff --shortstat v0.16.0 v0.17.3` = 525 fichiers, +73 620 / −2 195 ; code seul (`-- crates frontend
scripts`) : 354 fichiers, +41 846 / −1 881. Par release (code seul) : 0.17.0 +31 111 / −1 537 (269 fichiers) ; 0.17.1
+1 525 / −56 (44) ; 0.17.2 +5 255 / −567 (85) ; 0.17.3 +4 447 / −213 (66).

---

## 2. Les engagements, et la règle de release

```sh
gh issue list --state open --limit 500 --json number --jq length                       # 239
gh issue list --state open --limit 500 --json labels \
  --jq '[.[]|select([.labels[].name]|any(.=="bug" or .=="known-failure"))]|length'      # 87
gh issue list --state all --limit 1000 --json number,title,labels,state,createdAt,closedAt,milestone \
  > kesh-gate-logs/retro017/issues.json                                                   # 529 issues
python3 kesh-gate-logs/retro017/compte.py      # ouvertes, bugs et P1 à chaque tag
python3 kesh-gate-logs/retro017/apres.py       # fermetures dans les 30 min autour de chaque tag
```

Exécutées le 2026-11-05. Une issue est ouverte au tag si `createdAt ≤ tag` et `closedAt` est vide ou postérieur ; l'heure
du tag est `git for-each-ref refs/tags/<tag> --format='%(taggerdate:iso-strict)'`. **Une issue fermée par la fusion de la
PR de release compte comme fermée au tag** (précédent #792, rétrospective 32 § 2.1).

### 2.1 Le décompte

| Moment | Issues ouvertes | Bugs | P1 ouverts | Mouvement |
|---|---|---|---|---|
| Tag v0.16.0 (2026-10-28 10:57) | **185** | **66** | 14 | rétrospective 32 : 186 / 67 (+#809) |
| *v0.16.1-rc1* (2026-10-29 17:00) | 190 | 70 | 14 | pré-version |
| *v0.17.0-rc1* (2026-10-31 03:05) | 220 | 86 | 14 | pré-version |
| **Tag v0.17.0** (2026-11-03 14:58) | **233** | **88** | 13 | #789 fermée ; seize fermetures au merge de #846 |
| **Tag v0.17.1** (2026-11-04 10:35) | **236** | **90** | 13 | quatre fermetures au merge de #884 ; #854 fermée 21 min après le tag, sur le constat de Guy |
| **Tag v0.17.2** (2026-11-04 17:47) | **238** | **88** | 14 | +#890 ; huit fermetures au merge de #894 |
| **Tag v0.17.3** (2026-11-05 13:00) | **239** | **87** | 14 | #876, #891 fermées au merge de #900 |
| Aujourd'hui (2026-11-05) | 239 | 87 | 14 | |

**Sur la série** (`periode.py`, du tag v0.16.0 au tag v0.17.3) : **86 issues ouvertes, 32 fermées** ; 185 + 86 − 32 =
239. Parmi les 86 : 47 bugs, 20 `enhancement`, 18 `technical-debt`, 1 `documentation` ; 27 ont été ouvertes **et**
fermées pendant la série. Parmi les 32 fermées : 26 bugs ; cinq seulement sont antérieures au tag v0.16.0 (#671, #687,
#735, #746, #789).

⚠️ **Le tag v0.17.2 porte 238 issues, non 246 ni 243.** Cinq des huit fermetures du merge de #894 (#851, #880, #882,
#883, #885) sont horodatées **une à deux secondes après** le tag (16:47:47–48Z pour un tag à 16:47:46Z) : le décompte
strict par horodatage rend 243 / 92 ; la règle (fermées au tag) rend **238 / 88**, le nombre de l'orchestrateur. Le
« 246 » écrit par l'agent d'intégration de la 0.17.3 ne se retrouve par aucune des deux méthodes (§ 4.5).

⚠️ **Les décomptes écrits au `CLAUDE.md` sont ceux « à la préparation du tag »** (240, 244, 241), et le tag réel y est
reconstitué ou « attendu » : 233, 236, 238 — confirmés ici — et « 239 attendues » pour la v0.17.3, confirmé (239 / 87).
Le « 241 … dont 88 bugs » du paragraphe de tête vaut à la préparation, non au tag (§ 10, Q10).

### 2.2 Engagement 1 — le nombre de bugs baisse : ❌ **non tenu (66 → 87)**

Quarante-sept bugs ouverts, vingt-six fermés. **Quatrième période de suite sans baisse** (Epic 30 : 49 → 49 ; Epic 31 :
49 → 51 ; Epic 32 : 51 → 66 ; série 0.17 : 66 → 87). La cible proposée par la rétrospective 32 (Q4 : « moins de 66 à la
rétrospective suivante ») et la réponse de Guy (« j'aimerais réduire le nombre de bugs pour la prochaine release »,
relevé de l'orchestrateur, 2026-10-28) ne sont pas atteintes.

Ce que le chiffre cache, et qu'il faut dire avec lui :

- **Vingt-deux des quarante-sept bugs neufs sont déjà fermés** (#809, #811, #812, #824, #830, #831, #832, #835, #840,
  #851, #853, #854, #863, #866, #870, #871, #876, #878, #879, #880, #882, #883). Les vingt-cinq qui restent sont
  surtout de faible priorité : **13 P3, 9 P4, 2 P2 (#817, #819), 1 P1 (#890)**. Les défauts qui gênaient la tenue des
  livres ont été corrigés dans les jours qui ont suivi leur ouverture ; ce sont les constats secondaires qui s'empilent.
- **Quatre issues de LOW de revue portent le label `bug`** (#875, #883, #892, #893), alors que la convention de l'Epic 32
  les ouvrait en `technical-debt` : elles comptent dans l'engagement 1. Trois restent ouvertes. Le label d'une issue de
  LOW n'a pas de règle écrite (§ 10, Q2).
- **Les bugs anciens n'ont presque pas bougé** : quatre fermés sur les soixante-six du tag v0.16.0. Les quatorze P2
  ouverts (#431, #435, #450, #465, #486, #513, #540, #606, #729, #742, #755, #776, #817, #819) n'ont pas été touchés,
  hors #817 et #819 ouverts pendant la série.

### 2.3 La règle de release : ❌ **objectif manqué à chaque tag, d'un écart qui se resserre**

| Tag | Objectif (moins que le tag précédent) | Au tag | Écart |
|---|---|---|---|
| v0.17.0 | < 185 | 233 | +48 |
| v0.17.1 | < 233 | 236 | +3 |
| v0.17.2 | < 236 | 238 | +2 |
| v0.17.3 | < 238 | 239 | +1 |

La règle est un objectif à terme, non une barrière (`CLAUDE.md`, § *Priorités des défauts*, 2026-10-24) : Guy a publié
chaque fois. **Aucune issue n'a été fermée pour le chiffre** : les trente-deux fermetures portent chacune un correctif
fusionné, sauf #735, fermée *not planned* sur accord de Guy (obsolète : elle visait la PR #743, jamais fusionnée). La
hausse de la 0.17.0 vient de la recette en production (§ 5) et du traçage des LOW : **seize issues regroupent les LOW
d'une revue** (#814, #833, #834, #836, #839, #843, #845, #862, #864, #865, #875, #883, #892, #893, #898, #899 ; quinze
ouvertes, #883 fermée par la 0.17.2).

⚠️ **Un doublon** : #896 et #897 tracent le même geste (retirer le paramètre d'inactivité ignoré de
`Config::from_fields_for_test`), ouvertes à trois heures d'intervalle le 2026-11-04 par deux agents de la story
session. *« On trace toujours » ne dit pas de chercher avant d'ouvrir.* L'une est à fermer comme doublon (§ 8, A7).

### 2.4 Engagement 2 — aucun P1 ne passe d'une période à l'autre : ⚠️ **un fermé, un neuf reporté, un prévu non fait**

| Issue | Objet | Sort |
|---|---|---|
| **#789** | quick-xml 0.39, deux avis RustSec (DoS de l'import CAMT.053) | ✅ **fermée par la 0.17.0** (story v017-1, première de la série, comme la rétrospective 32 le demandait) |
| **#890** | **sécurité** : le jeton de renouvellement est stocké **en clair** (`refresh_tokens.rs:30-33`), le manuel d'administration affirmait un hash SHA-256 | ouverte le 2026-11-04 pendant la fiche de la 0.17.2 (C-v0172b-25) ; **seul le texte du manuel a été corrigé** (0.17.2) ; **reportée**, sans décision écrite sur sa date |
| **#235** | pièces justificatives sur les écritures (art. 957a CO) | **prévue en 0.17.0** par Guy le 2026-10-24 ; **non faite** (§ 1.1) |
| **#404** | aucun rapport ne franchit la borne d'exercice | epic pluriannuel dédié (décision du 2026-10-24), inchangé |
| **#98** | signature électronique qualifiée (CO 958f) | `v0.2-milestone`, inchangé |
| **Les dix P1 de la TVA** (#390 à #397, #401, #525) | jalon « Vague 2 » | **report assumé** (décision du 2026-10-08) : le seul utilisateur réel n'est pas assujetti |

*(`compte.py` rend ces quatorze P1 au tag v0.17.3, et eux seuls.)* **#890 mérite une remarque que l'issue ne fait pas** :
la 0.17.3 porte la durée d'inactivité par défaut de 15 à **60 minutes** (CR #891). Un jeton de renouvellement lu dans
une copie de la table `refresh_tokens` reste donc utilisable plus longtemps qu'au moment où #890 a été écrite (« risque
atténué par l'expiration courte »). La 0.17.3 révoque les jetons restaurés par un import complet, ce qui ferme le cas de
la restauration, non celui d'une copie de la base ou d'un `.keshbackup` lu hors de Kesh. *C'est un raisonnement sur le
code et le CHANGELOG, non un essai* (§ 10, Q3).

### 2.5 Engagement 3 — une fonctionnalité nouvelle : ✅ **tenu**

Le relevé saisi à la main (#816), le rattachement à une écriture existante (#671), le cycle brouillon → validée → payée
des factures fournisseur (#849) et leur retour en brouillon (#868), la durée de session réglable (#891).

---

## 3. Chiffres de processus

### 3.1 Validations — 72 passes

`ls kesh-gate-logs | grep -E '^v01[0-9a-z-]*-validate-[pr][0-9]+(-[A-Z])?\.md$' | sed -E 's/-validate-([pr][0-9]+).*/ \1/' | sort -u | wc -l`
= **72** couples (fiche, passe), en 96 rapports. Notation CRITICAL/HIGH/MEDIUM/LOW, telle qu'écrite aux Change Logs, au
registre de sprint ou au relevé de l'orchestrateur ; « R+F » = somme des deux lentilles.

| Fiche | Passes | Trend | Sortie |
|---|---|---|---|
| v016-1 | 6 | au-dessus de LOW : 8 → 6 → 1 → 3 → 1 → 1 (P1 Sonnet 1 HIGH, 7 MEDIUM) | **close sur décision de Guy** (« v016-1 validé », 2026-10-29), P6-1 (MEDIUM) renvoyé à la préparation de la release ; signal de découpage en P4 → recette sortie de la fiche vers une candidate rc |
| v016-2 (parente) | 2 | P1 Sonnet 0/0/12/17 → P2 Opus 0/0/10/17 | **découpée** en 2a et 2b par l'orchestrateur (C-v016-2-18), approuvée par Guy |
| v016-2a | 3 | P3 0/0/2/14 → P4 0/0/1/7 → P5 0/0/0/5 | close |
| v016-2b | 6 | P3 0/0/1/21 → P4 0/0/1/9 → P5 0/0/2/5 → P6 0/0/2/4 → P7 0/0/2/3 → P8 0/0/1/3 | **close au plafond**, MEDIUM renvoyé à l'étape 0 du développement (règle du 2026-10-24) |
| v017-1 | 4 | P1 4 MEDIUM → P2 4 → P3 1 → P4 ciblée Opus 0/0/0/4 | close |
| v017-2 | 3 | P1 R 0/0/3/4, F 0/0/2/8 → P2 0/0/2/13 → P3 ciblée 0/0/0/7 | close |
| v017-3a | 5 | P1 R 0/0/7/6, F 0/1/5/6 → P2 R 0/0/4/6, F 0/0/8/8 → P3 R 0/0/2/6, F 0/0/4/6 → P4 0/0/1/8 → P5 0/0/0/6 | close ; **découpée en P2** (3a dans la 0.17.0, 3c en `backlog`) ; recyclage déclaré à Guy |
| v017-3b | 6 | P1 R 0/0/5/7, F 0/0/6/7 → P2 R 0/0/5/7, F 0/0/6/10 → P3 R 0/0/0/9, F 0/0/1/9 → P4, P5 ciblées → P6 ciblée Opus 0/0/0/4 | close ; **simplifiée en P2** (avertissement statique au lieu d'une matrice de conduite) |
| v017-4 | 5 | P1 R 0/0/6/5, F 0/0/6/11 → … → P5 0/0/0/6 | close ; découpée dès la rédaction (4a, 4b) |
| v017-5 | 5 | P1 8 MEDIUM → P2 1 HIGH, 10 MEDIUM → P3 2 → P4 3 → P5 ciblée 0/0/0/5 | close |
| v017-6 | 4 | P1 Opus 0/2/4/6 → P2 Sonnet 0/0/2/8 → P3 Opus 0/0/2/6 → P4 Sonnet 0/0/0/4 | close |
| v0171-correctifs | 4 | Opus 0/0/5/8 → Sonnet 0/0/1/4 → ciblée Opus 0/0/1/3 → ciblée Sonnet 0/0/0/3 | close |
| v0172-annulees | 5 | 0/0/7/8 → 0/1/4/8 → 0/0/2/5 → 0/0/3/4 → 0/0/0/4 | close |
| v0172-correctifs | 5 | 0/0/12/10 → 0/0/11/12 → 0/0/4/5 → 0/0/1/6 → 0/0/0/3 | close ; **#876 extraite en P2** vers la fiche session |
| v0172-session | 9 | P3 0/0/7/10 → P4 0/0/7/13 → P5 0/1/4/9 → P6 0/0/3/13 → *(P7 arrêtée)* → **conception redéfinie par Guy** → R1 0/0/5/12 → R2 0/0/3/12 → R3 0/0/2/7 → R4 0/0/2/6 → R5 0/0/0/7 | close (§ 3.3) |

**Volume** : 72 passes pour quinze fiches livrées et trois parentes (69 à l'Epic 32 pour treize fiches). **Une fiche au
plafond** (v016-2b), **une close par le Project Lead avant le plafond sur un MEDIUM** (v016-1), **quatre découpages ou
simplifications en P2** (v016-2, v017-3a, v017-3b, v0172-correctifs). Les fiches font **13 796 lignes** en tout (`wc -l`
des dix-huit fiches ; de 116 lignes pour la v017-3c à 1 289 pour la v017-3a), pour 19 717 à l'Epic 32.

⚠️ **La clôture de la v016-1 n'entre dans aucune des règles écrites** : la boucle ne se clôt que sur une passe sans
MEDIUM, ou au plafond de huit passes ; elle s'est close à la sixième, sur un MEDIUM non remédié (P6-1, procédure de
release), par la parole de Guy. Ce n'est pas une faute — le Project Lead peut lever une règle qu'il a posée —, mais la
dérogation n'est écrite qu'au Change Log de la fiche et au journal, non sous une forme que la prochaine boucle
retrouvera (§ 10, Q9).

### 3.2 Revues de code — 28 passes

| Story | Passes | Trend |
|---|---|---|
| v016-1 | 3 | Sonnet 0/0/2/13 → Opus 0/0/3/14 → ciblée Sonnet 0/0/0/4 |
| v016-2a | 2 | Sonnet 0/0/2/14 → ciblée Opus 0/0/0/7 |
| v016-2b | 3 | Sonnet 0/0/4/18 → ciblée Opus 0/0/1/6 → ciblée Sonnet 0/0/0/3 |
| v017-1 | 1 | Sonnet B 0/0/0/4, E 0/0/0/2, A 0/0/0/4 |
| v017-2 | 1 | Sonnet B 0/0/0/3, E 0/0/0/4, A 0/0/0/3 |
| v017-3a | 1 | Sonnet 0/0/0/15 |
| v017-3b | 1 | Sonnet 0/0/0/14 |
| v017-4a | 2 | Opus 0/0/6/12 → Sonnet 0/0/0/4 |
| v017-4b | 3 | Opus 0/0/5/23 → Sonnet 0/0/4/16 → ciblée Opus 0/0/0/4 |
| v017-5 | 2 | Opus 0/0/1/14 → Sonnet 0/0/0/4 |
| v017-6 | 1 | Sonnet 0/0/0/12 |
| v0171-correctifs | 3 | Sonnet B 0/0/0/3, E 0/0/0/3, A 0/0/1/4 → ciblée **Haiku** 0/0/1/2 → ciblée Sonnet 0/0/0/1 |
| v0172-annulees | 1 | Sonnet B 0/0/0/4, E 0/0/0/4, A 0/0/0/5 |
| v0172-correctifs | 2 | Opus B 0/0/0/9, E 0/0/0/8, A 0/0/1/8 → ciblée Sonnet 0/0/0/1 |
| v0172-session | 2 (par lot S et N) | Sonnet S 0/0/2/≈18 (+ S-B1 reclassé MEDIUM), N 0/0/4/≈20 → Opus S 0/0/0/10, N 0/0/0/6 |

**Volume** : `… -review-[pr][0-9]+ … | sort -u | wc -l` = **28 passes** en 70 rapports (50 passes à l'Epic 32). **Six
stories closes dès la P1**, aucune au-delà de trois passes, **aucun HIGH**. Toutes closes sur une passe à 0 au-dessus de
LOW. **Haiku : une passe, ciblée** (v0171, P2) ; la règle « une première passe n'est jamais ciblée » est tenue.

**Rapports hors dépôt de la série** : 96 + 70 = **166**.

### 3.3 La session (#876) — quand la conception tourne en rond

C'est l'histoire la plus instructive de la série.

1. **Le défaut** (#876, ouvert le 2026-11-03, P2) : la session se fermait quinze minutes après la connexion, en pleine
   saisie, la saisie perdue. Guy le subissait sur ses livres.
2. **Première conception** (fiche v0172-correctifs, sous-story A) : renouvellement anticipé du jeton d'accès. Validation
   P1 et P2 : 12, puis 11 MEDIUM sur la fiche commune, dont six nés de la remédiation et la plupart sur #876 ; **signal de
   découpage déclaré à Guy**, #876 extraite en fiche sœur (relevé de l'orchestrateur, 2026-11-04).
3. **Deuxième conception** (P3 : 7 MEDIUM sur les verrous, les onglets, la suspension) : l'orchestrateur demande à
   l'agent d'évaluer d'abord une conception simple ; elle est **substituée** (C-v0172s-15 à 21 : inactivité de 60 min,
   N ≥ 2 × la durée du jeton d'accès, grâce de rotation, repli `/auth/me`). P4 : **7 MEDIUM encore** — tendance
   « 7 → 11 → 7 → 7 » au relevé de l'orchestrateur —, et la P4 trouve que **FR13 du PRD fixe quinze minutes** : la
   conception demandait un CR.
4. **Questions à Guy** au lieu d'une cinquième remédiation : #876 sort de la 0.17.2 (« 1 ok ») ; 60 minutes (« 2 : 60
   min ») → **CR #891**. P5 (0/1/4/9) et P6 (0/0/3/13) trouvent encore un HIGH (motif de révocation refusé par une
   contrainte `CHECK`) et un contournement (480 → 960 min par la règle du double).
5. **Troisième conception, par Guy** : la durée d'inactivité se règle dans l'administration, **exacte** (sans règle du
   double), remise à zéro par toute activité. La P7 est arrêtée, la fiche reprise par un agent neuf, la validation
   **repart à R1**.
6. **Convergence nette** : R1 5 → R2 3 → R3 2 → R4 2 → R5 0 MEDIUM ; revue de code P1 (6 MEDIUM sur deux lots) → P2 0.

**Six passes de validation avant la redéfinition (la septième arrêtée), cinq après.** Avant, chaque passe trouvait ce que la remédiation
venait d'ajouter à une conception qui empilait les mécanismes (renouvellement anticipé, puis règle du double, puis
contournement) ; après, la conception était **celle que l'utilisateur voulait**, énoncée par lui, et la boucle a
convergé de façon monotone. *Quand la sévérité stagne et que les MEDIUM portent sur la conception elle-même, la passe
suivante n'est pas le bon outil : c'est une question au Project Lead.* La règle de découpage (D5) le disait à moitié —
elle déclare le signal à Guy ; elle ne dit pas de lui poser **la question de conception** (§ 7.3, leçon 1).

### 3.4 Le registre des choix

**383 choix autonomes distincts** dans les dix registres de la série
(`grep -ohE '^#{2,4} C-v0[0-9a-z-]+-[0-9]+' v01*-choix-autonomes.md | sed -E 's/^#+ //' | sort -u | wc -l`) : v016-1
19, v016-2 28, v017-1 18, v017-2 17, v017-3 41, v017-4 70, v017-5 19, v017-6 14, v0171 25, v0172 132 (dont 61 pour la
session). Pas de registre d'epic où les verser — la série n'en a pas : les registres sont par release ou par story,
tenus par union (`docs/agents-orchestration.md` § 1).

### 3.5 Gates, mutations, parallélisme

Gate complet et E2E complet au dernier commit de code de chaque story et de chaque branche d'intégration (Change Logs,
journaux `rel01*`) ; partout les sept KF-029. Mutations jouées à chaque story (de 7 pour la v017-1 à 40 pour la session ;
le total n'est pas additionné, les Change Logs les comptant sous des formes différentes ; le « 42 (21 S) » écrit au
développement de la session a été recompté à 40 en revue de code). **Jusqu'à quatre fiches en validation** (v016-2, v017-1, v017-2,
v017-3), **puis quatre développements en parallèle** (v017-1, v016-2a, v017-3b, v017-3a), le 2026-10-30, chacun dans son worktree, son port
(3161 à 3176) et ses bases. **Deux migrations dans la 0.17.0, une dans la 0.17.3**, jamais deux en développement à la
fois. Un **démarrage depuis une base peuplée par l'image précédente** à chaque release (§ 5).

---

## 4. Incidents

### 4.1 La mémoire de la station : deux sessions perdues

`journalctl -u systemd-oomd --since "2026-10-28"` relève **quatorze** interventions d'oomd dans la série. Douze ont tué
ce qu'elles devaient : des gates sous plafond `mem-guard` (huit de mybibli, un de kesh le 2026-10-31) et Firefox (trois).
**Deux ont tué des sessions de travail** :

- **2026-11-04, 21:48:42 — GNOME Shell** (`org.gnome.Shell@ubuntu.service`, pression de la tranche utilisateur à 56,75 %),
  c'est-à-dire **toute la session graphique**, pendant l'E2E r2 de la remédiation P1 de la session, alors qu'un gate
  mybibli tournait (relevé de l'orchestrateur). L'E2E, coupé vers le test 190, a été rejoué (r4) ; le code était commité.
- **2026-11-05, 10:58:12 — un onglet de terminal** (`ptyxis-spawn-…`, 83,11 %), sans gate kesh en cours.

**Le motif** : deux places `mem-guard` de 10 Go chacune sur une station de 30 Go bornent chaque gate, non leur somme avec
le reste de la station (navigateur, terminaux, shell). Le 2026-11-04 seul compte six interventions (13:46 à 21:48).
**Le remède a été convenu, non outillé** : un **verrou commun `/tmp/station-heavy.lock`**, accordé le 2026-11-05 avec la
session mybibli — un seul gate lourd sur toute la station, l'autre session prévenue à la libération (relevé de
l'orchestrateur). Il a été **tenu sans incident jusqu'au tag v0.17.3** : aucune intervention d'oomd après 10:58 le
2026-11-05. Le verrou n'est écrit qu'au Change Log de la session (`v0172-session.md:1102`) et dans le prompt
d'intégration de la 0.17.3 ; **ni le `CLAUDE.md`, ni `docs/agents-orchestration.md`, ni `scripts/mem-guard.sh` ne le
connaissent** (`git grep station-heavy` ne rend que la ligne de la fiche) — § 10, Q4.

Un **troisième arrêt** n'est pas un OOM : le 2026-10-28 vers 15 h 10, crash de la session et redémarrage de la station
(cause non établie au journal) ; reprise après contrôle d'intégrité (git, YAML, `.env`, mémoires), base de dev à
refaire (tmpfs vide).

### 4.2 Un verrou tenu par un orphelin (2026-11-04)

Un backend E2E de la v0172-annulees, lancé en arrière-plan, a **hérité du descripteur du `flock`** et tenu
`.gate.lock` **36 minutes** après la fin de son script (relevé de l'orchestrateur). Arrêté par PID ; le run E2E
concerné, invalide, a été rejoué ; le script corrigé (`flock -o`, ou fermeture du descripteur `9>&-` avant de lancer le
binaire). C'est la **deuxième fois** qu'un backend E2E survit à son script (rétrospective 32, § 4.3 : un `$!` qui visait
un sous-shell). La consigne n'est pas versée (§ 6, A1).

### 4.3 Une limite de dépense, et une reprise sur un WIP

Le 2026-10-31, Guy arrête le travail (limite de dépense de l'API), reprise le 2026-11-03. La remédiation P1 de la
v017-4b, interrompue à mi-course, avait été **mise à l'abri dans un commit WIP** (`165d9559`, « non vérifié ») ; un agent
neuf l'a reprise sur un prompt de remédiation, puis gate et E2E complets. **Le point de reprise écrit au journal**
(branches, commits, arbitrages déjà donnés, mandats) a suffi : aucune perte.

### 4.4 Écarts des agents (déclarés)

- **Un agent lancé sur un prompt mal dérivé** (v017-3a, 2026-10-30 : un `Write` échoué, le fichier non relu) — arrêté
  aussitôt, rien écrit, relancé sur le bon prompt.
- **Un script de restauration des mutations a effacé un Dev Agent Record non commité** (v017-3a), réécrit.
- **Tests rouges non commités séparément, éditions par script** (v016-2a), déclarés.
- **Un compteur faux après une fusion sans conflit** (intégration de la 0.17.0) : `traced` resté à 130 pour 131 réels ;
  le recompte depuis la source, exigé par le prompt d'intégration, l'a trouvé. *Une fusion sans conflit ne garantit pas
  un compteur juste* — `git rerere` était actif.
- **Une capture de recette portait une clé MCP en clair** (`kesh-gate-logs/3211/capture-recette.log`, clé révoquée),
  contre la consigne « Bearer jamais cité » ; signalé à Guy, effacée le 2026-10-29 avec deux autres fichiers.

### 4.5 Deux comptes rendus faux, attrapés à la v0.17.3

L'agent d'intégration de la 0.17.3 a écrit deux affirmations qui n'avaient pas été vérifiées ; l'orchestrateur les a
relues avant la PR :

1. **« 246 issues au tag v0.17.2 »** : 238 au recompte (§ 2.1). Le nombre a été corrigé dans la PR #900 et au
   `CLAUDE.md` avant fusion.
2. **« Une sauvegarde `.keshbackup` prise en 0.17.2 s'importe dans la 0.17.3 »**, écrite au CHANGELOG et aux notes de
   version, **sans avoir tourné**. L'orchestrateur l'a ensuite éprouvée : import HTTP 200 (`rel0173/import-0173-reponse.txt` :
   `sourceVersion 0.17.2`, 43 tables, `sessionInvalidated true`), et les empreintes des tables comptables et des
   factures identiques à celles d'avant (`diff boot-avant.md5 boot-import.md5` ne diffère que sur `migrations.txt`, qui
   porte la ligne de la migration 89, attendue).

*Une phrase des notes de version est une promesse à l'administrateur qui met à jour ses livres.* Elle se joue comme un
critère d'acceptation, ou elle ne s'écrit pas (§ 7.3, leçon 3).

### 4.6 Le classifieur de permissions

Plusieurs écritures extérieures ont été refusées à l'orchestrateur et faites par Guy : la fermeture de la PR #815, le
retrait de sept worktrees, l'édition de la release GitHub (relevé de l'orchestrateur). `gh pr edit` échoue sur ce dépôt
(*Projects classic*) : le corps des PR passe par `gh api`. Coût faible, mais chaque refus est un aller-retour avec Guy.

---

## 5. Ce que les releases ont appris

- **Kesh est en production depuis le 2026-10-31**, et la série l'a vécu avant de l'écrire. Guy a déclaré ce jour-là
  avoir commencé à saisir des données réelles sur le NAS, passé **de la 0.16.1-rc1 à la 0.17.0-rc1 sans sauvegarde
  préalable** (« c'est une erreur de ma part, il faut intégrer les sauvegardes dans le processus de mise à jour ») →
  **#847** (P2, Kesh fait lui-même une sauvegarde complète avant d'appliquer des migrations), renvoyée par Guy à une
  release ultérieure. **Le `CLAUDE.md` a dit « aucun livre réel n'est tenu » du 2026-10-31 au 2026-11-03** ; il a été
  récrit dans la PR de la 0.17.0 (§ *Kesh est en production*). Trois jours où les règles qui s'appuient sur ce paragraphe
  (priorité des défauts de sauvegarde, prudence des migrations) étaient lues dans leur version d'avant.
- **La 0.17.0 finale n'a pas eu de rc2.** Elle porte quatre stories (v017-4a, v017-4b, v017-5, v017-6) et une migration
  (`20261031000001_supplier_invoices_draft`) qu'aucune candidate n'a portées, et elle est allée sur le NAS en production
  le 2026-11-03. Ce qui l'a rendu sûr : le **démarrage du binaire final sur une base peuplée par l'image 0.17.0-rc1**
  (`rel017/boot/boot.sh` ; écritures identiques avant et après) et l'**import du `.keshbackup` rc1** (relevé de
  l'orchestrateur). La sauvegarde du NAS avant cette mise à jour **n'est pas consignée** au journal.
- **Quatre releases en trois jours** (2026-11-03 au 2026-11-05) : la 0.17.1 le lendemain de la 0.17.0, sur la décision
  de Guy (« oui, fais une 0.17.1 »), la 0.17.2 le même jour qu'elle, la 0.17.3 le lendemain. Chacune naît d'un défaut qui gênait la
  tenue des livres : la QR-facture à lignes CR+LF d'un émetteur réel non lue (#854), les quantités et les dates
  illisibles, une facture annulée qu'on ne pouvait plus corriger, la session perdue en pleine saisie. **La règle « une
  release corrective quand un bug gêne la production » a été appliquée sans être écrite** (`git grep` n'en trouve
  trace ni au `CLAUDE.md` ni dans `docs/`) — § 10, Q5.
- **Les correctifs se diagnostiquent sur les pièces réelles, en local, et se testent sur des fixtures synthétiques** :
  #854 a été diagnostiquée sur deux pièces de Guy (relevé de l'orchestrateur, 2026-11-03), reproduite par une fixture,
  et fermée seulement **après le constat de Guy** sur ses pièces redéposées (2026-11-04). Aucune donnée réelle n'est
  entrée au dépôt.
- **Le refus de l'image précédente a enfin été joué** (A4 de la rétrospective 32, A6 de la 31, A9 de la 30) — à la
  0.17.3, première release à migration après la 0.17.0 : base 0.17.2 peuplée par l'image publiée, migrée par le binaire
  0.17.3, **refus constaté de l'image 0.17.2** (`rel0173/boot-0172-refus-container.log` : « migration 20261104000001
  was previously applied but is missing in the resolved migrations »), dump restauré, redémarrage de la 0.17.2,
  **empreintes identiques** (`diff boot-avant.md5 boot-retour.md5` vide). Les 0.17.1 et 0.17.2, sans migration, ont
  joué le **retour** à la version précédente sans restauration (`rel0171/boot-0170-retour-container.log`,
  `rel0172/boot-0171-retour-container.log`). ⚠️ **À la 0.17.0, qui portait deux migrations, le refus de la 0.16.0 n'a
  pas été joué** : l'échéance de l'action (« release 0.17.0 ») est donc manquée, et l'action tenue une release plus tard.
  Et elle vit dans des scripts de `kesh-gate-logs/rel0173/` (`boot-phase1.sh`, `boot-phase2.sh`), **non dans un script
  du dépôt** : rien ne la rejouera à la prochaine release si le prompt ne la redemande pas (§ 8, A3).
- **La recette Claude Code a été jouée à moitié.** Le 2026-10-28, sur la 0.16.0 et une base neuve, Guy et
  l'orchestrateur ont branché Claude Code 2.1.312 : branchement, outils, prompts observés ; **#811** et **#812** trouvés
  (`serveur-mcp-decisions.md`) — la recette a donc rendu ce qu'on attendait d'elle. La re-recette sur la candidate (C1 à
  C3, `rel0161/recette.md`) n'a **pas été jouée** : reportée par la reconstruction des disques du NAS, puis la 0.17.0
  est sortie avec ses réserves (décision de Guy du 2026-11-03). Le manuel d'administration porte encore **huit**
  « non éprouvé », le CHANGELOG deux (`git grep -c 'non éprouvé'`), le manuel utilisateur aucun.
- **Le défaut de stockage du jeton de renouvellement** (#890) a été vu par une fiche, non par une revue : en lisant le
  manuel contre le code (C-v0172b-25). *Le manuel, une fois de plus, est l'endroit où l'écart se voit*
  (`CLAUDE.md`, § *Le prompt d'une passe doit NOMMER le manuel*).
- **La date de publication de la 0.17.1 est fausse d'un jour** : `CHANGELOG.md` (`## [0.17.1] — 2026-11-03`) et
  `\keshReleaseDate{2026-11-03}` (`git show v0.17.1:docs/manual/shared/kesh-style.sty`), alors que le tag est du
  2026-11-04 à 10:35 et la release GitHub publiée le 2026-11-04 à 09:44Z (`gh release view v0.17.1`). La règle 4-bis
  veut la date de publication. La fiche a été écrite le 2026-11-03, la release a passé minuit. Les PDF publiés de la
  0.17.1 portent la date fausse ; le CHANGELOG se corrige, un PDF publié non (§ 8, A7).

---

## 6. Suivi des actions de la rétrospective de l'Epic 32 (§ 8 et § 10)

| # | Action | État au 2026-11-05 | Preuve |
|---|---|---|---|
| A1 | Verser à `docs/agents-orchestration.md` les consignes nées pendant l'Epic 32 (nettoyage après `MERGED` constaté et `&&`, `grep -cF` sous `timeout 60`, aucun `rm`, lentilles sans redirection, processus d'arrière-plan suivi par son PID) | ❌ | `git log v0.16.0..HEAD -- docs/agents-orchestration.md` est vide. Les consignes vivent toujours dans les prompts hors dépôt ; l'orphelin du `flock` (§ 4.2) en est la récidive |
| A2 | Verser les quatorze registres de l'Epic 32 (618 choix), aligner les neuf *Status* ; puis faire porter ces gestes par la PR de chaque story | ⏳ | **Pour l'Epic 32 : non fait** — `epic-32-choix-autonomes.md` porte toujours 13 choix (`grep -cE '^#{2,4} C-32-'`), les neuf fiches `32-*` gardent `ready-for-dev` ou `review`. **Pour la série 0.17 : fait** — les quinze fiches livrées portent `Status: done`, la v017-3c `backlog`, les trois parentes `split` (`grep -m1 '^Status:'`) : le geste est entré dans les prompts de clôture |
| A3 | Recette Claude Code (AC-A8) ; retirer les « non éprouvé » ; **ouvrir l'issue qui la trace** | ⏳ | Recette jouée le 2026-10-28 (#811, #812 trouvés, fermés par la 0.17.0) ; re-recette C1-C3 non jouée ; **aucune issue ne la trace** (#723, le CR du serveur MCP, porte le bilan en commentaire) ; huit « non éprouvé » au manuel d'administration |
| A4 | Le refus de l'image précédente : l'inscrire au script de gate de release, ou l'abandonner par écrit | ⏳ | **Joué à la 0.17.3** (§ 5), **non à la 0.17.0** (l'échéance) ; scripts dans `kesh-gate-logs/rel0173/`, **aucun script du dépôt** (`git grep "previously applied" -- scripts` vide) |
| A5 | #789 en tête | ✅ | Story v017-1, la première de la série ; #789 fermée par la 0.17.0 ; `cargo audit` sans avis quick-xml (registre de sprint) |
| A6 | Story courte des défauts du parcours (#803, #804, #805) avec #809 | ⏳ | **#809 fermée** (v017-2, 0.17.0) ; **#803, #804, #805 ouvertes** |
| A7 | Vérifier DSM et `X-Forwarded-For` par le journal du filtre à la première activation du MCP sur le NAS | ⏳ | Non consigné : la recette du 2026-10-28 passait par un relais local ; aucune ligne `mcp_filtre` relue n'est au journal |
| A8 | Nettoyer les 62 `/tmp/kesh-e2e-*` ; rendre active la règle de permission | ⏳ | Les 62 ont disparu (`/tmp/kesh-e2e-313` n'existe plus ; la station a redémarré le 2026-10-28, la cause de leur disparition n'est pas établie) ; **32 nouveaux** du 2026-11-03 au 2026-11-05 (`ls -d /tmp/kesh-e2e-* \| wc -l`) : les agents ne nettoient toujours pas |
| A9 | Axe « la donnée rendue est-elle juste » dans les prompts de revue des outils MCP | ❌ | Aucun prompt de la série ne le nomme (`grep -rl "donnée rendue"` vide sur `kesh-gate-logs/prompts/` et les prompts versionnés) ; la seule story MCP de la série (v016-1) portait sur le canal de sortie, non sur les valeurs |
| Q1 | Livres réels sur le NAS ? | ✅ répondue | « j'aimerais bien commencer à tenir des livres réels » (2026-10-28), puis « Kesh doit être considéré comme en production » (2026-10-31) ; `CLAUDE.md` récrit le 2026-11-03 |
| Q2 | Quand la recette Claude Code ? | ✅ répondue, ⏳ à moitié jouée | « je veux faire la recette claude code dès maintenant » (2026-10-28) ; voir A3 |
| Q3, Q4 | Objectif d'issues ; cible de bugs | ✅ répondues, ❌ non tenues | « j'aimerais réduire le nombre de bugs pour la prochaine release » ; 185 → 233 au tag v0.17.0, bugs 66 → 88 |
| Q5 à Q10 | Refus de l'image ; points de fiches ; questions de la rétrospective 31 ; `/tmp` ; ordre de l'Epic 33 ; longueur des fiches | — | **Sans réponse consignée** au journal. Q9 (ordre de l'Epic 33) est devenue sans objet (§ 1.1) |

**Bilan, sur neuf actions** : 1 faite (A5), 6 en cours ou à moitié (A2, A3, A4, A6, A7, A8), 2 non faites (A1, A9).
**Ce qui a été fait, c'est ce qu'une story ou un prompt portait** (A5 : la story v017-1 ; A2 pour la série : la ligne
du prompt de clôture ; A4 : le prompt d'intégration de la 0.17.3) ; ce qui ne l'a pas été n'avait pas de porteur — le
constat de la rétrospective 32 (§ 7.3, leçon 4), pour la quatrième fois.

---

## 7. Ce qui a marché, ce qui a coûté, les leçons

### 7.1 Ce qui a marché

- **Kesh a tenu des livres réels pendant toute la série, et rien n'a été perdu.** Quatre mises à jour du NAS consignées
  (0.16.1-rc1, 0.17.0-rc1, 0.17.0, 0.17.1), dont deux à migration, sans incident de données relevé.
- **La vitesse de correction quand un défaut gêne la production** : vingt-deux bugs ouverts et fermés dans la série ;
  la 0.17.1 publiée le lendemain de la 0.17.0, la 0.17.2 le même jour ; #854 fermée sur le constat de Guy.
- **Le démarrage depuis une base peuplée par l'image précédente, à chaque release**, et le refus de l'image précédente
  joué à la 0.17.3 : la procédure de retour écrite au CHANGELOG est éprouvée avant d'être écrite — sauf la phrase de
  l'import (§ 4.5), éprouvée après.
- **L'intégration par une branche `release/0.17.x` et un agent dédié** : quatre PR au lieu d'une par story, un seul
  CHANGELOG à réunir, les compteurs de gardes recomptés après chaque fusion (et un compteur faux trouvé ainsi, § 4.4),
  les PDF régénérés une fois.
- **Le découpage et la simplification en P2** (v016-2, v017-3a, v017-3b, v0172-correctifs) et la **question à Guy** à la
  P4 de la session : les boucles longues ont été coupées plus tôt qu'à l'Epic 32.
- **Les revues de code** : 28 passes pour quinze stories, six closes en P1, aucun HIGH, aucune au-delà de trois passes.
- **Le verrou commun de station**, une fois posé, a tenu.
- **L'audit des issues ouvertes** (2026-11-03, trois agents en lecture seule, `rel017/audit-issues-*.md`) : 242 issues
  lues contre le code (81 + 80 + 81) ; **aucune fermable hors des fermetures de la 0.17.0**, sinon #746 (fermable une fois
  `docs/manual/README.md:115` corrigé, fait à la préparation, fermée au merge) et #735 (obsolète, fermée *not planned*
  sur accord de Guy) ; trois de la tranche haute (#824, #832, #863) confirmées couvertes par la 0.17.0 ; treize
  « partielles » laissées ouvertes (#125, #126, #227, #252, #265, #540, #562, #722, #723, #761, #781, #796, #801). *Le résultat est un bon signe sur la discipline des fermetures — rien ne traînait
  d'implémenté —, et un mauvais sur la pile : elle est réelle.*

### 7.2 Ce qui a coûté

- **La pile d'issues** : +54 net (185 → 239), +21 bugs ; l'objectif manqué à chaque tag.
- **La session** : onze passes de validation menées à leur terme (une douzième arrêtée) et trois conceptions pour un
  défaut P2 (§ 3.3).
- **La mémoire de la station** : deux sessions perdues, un E2E à rejouer ; une demi-journée de gates sous la contrainte
  d'un seul gate lourd à la fois sur toute la station.
- **La série a pris la place de l'Epic 33** : les propositions de l'agent et #235 (P1) attendent toujours (§ 1.1).
- **Des PR de release grandes** : la #846 porte +31 111 lignes de code et onze stories ; une relecture humaine d'une
  telle PR n'est pas réaliste — la qualité repose entièrement sur les revues par story et sur les gates de la branche
  d'intégration.
- **Des gestes de clôture sans porteur**, encore (§ 6).

### 7.3 Leçons

1. **Quand une validation tourne en rond sur la conception, la passe suivante n'est pas le bon outil : c'est une
   question au Project Lead.** La session a convergé en cinq passes dès que Guy a énoncé la conception qu'il voulait,
   après six passes sur des conceptions qui empilaient des mécanismes. Le signal de découpage (D5) se déclare déjà à
   Guy ; il faut y joindre **la question de conception** — « que doit faire Kesh ? » —, et ne pas relancer de passe tant
   qu'elle n'a pas de réponse.
2. **Un geste de clôture ne se fait que s'il a un porteur** — une story, un prompt, un script. La série le confirme
   dans les deux sens : ce qui était dans un prompt a été fait (Status, refus de l'image à la 0.17.3), ce qui n'y était
   pas ne l'a pas été (consignes versées, registres de l'Epic 32, axe A9).
3. **Une phrase des notes de version est un critère d'acceptation.** Elle se joue, ou elle ne s'écrit pas ; et le
   décompte au tag se recompte **après** la fusion, fermetures du merge comprises, par une commande citée.
4. **Plafonner chaque gate ne borne pas la station.** Seule une règle sur la somme — le verrou commun — l'a fait. Elle
   tient tant qu'elle est respectée par toutes les sessions ; elle ne vaut donc que si elle est écrite là où toutes les
   lisent.
5. **La production fait le plan.** Une série qui suit les constats de l'utilisateur livre ce qui sert, mais elle
   repousse ce qui était décidé (#235, P1). Il faut que ce report soit **décidé**, non subi : c'est une question du § 10.

---

## 8. Actions

| # | Action | Propriétaire | Échéance | Critère |
|---|---|---|---|---|
| A1 | **Verser les consignes au dépôt** — celles de l'Epic 32 (A1 de la rétrospective 32, reportée) et celles de la série : le verrou commun de station si Guy l'accepte (Q4), le backend E2E lancé par son propre PID avec le descripteur du `flock` fermé (`9>&-`), le recompte des compteurs après toute fusion, même sans conflit | Orchestrateur | Avant la prochaine story | `docs/agents-orchestration.md` porte ces sections ; les prompts communs y renvoient au lieu de les recopier |
| A2 | **#890 en tête de la prochaine release** (sous réserve de Q3) : empreinte du jeton, recherche par empreinte, invalidation à la mise à jour, sauvegarde et export sans jeton utilisable, manuel aligné | Orchestrateur | Prochaine release | #890 fermée ; un test qui lit `refresh_tokens` et n'y trouve aucun jeton utilisable |
| A3 | **Inscrire au dépôt le test de montée et de retour de version** — base peuplée par l'image publiée précédente, migration par le binaire neuf, refus de l'image précédente cherché au journal (`previously applied`), restauration, empreintes — en reprenant les scripts de `kesh-gate-logs/rel0173/` | Orchestrateur | Prochaine release qui porte une migration | Un script du dépôt (`scripts/`) que le prompt de release lance ; journal cité dans la PR |
| A4 | **Les phrases des notes de version se jouent** : le prompt d'intégration liste chaque phrase du CHANGELOG et des notes qui affirme un fait vérifiable, avec le journal qui la prouve ; le décompte au tag se recompte après la fusion par la commande du § 2 | Orchestrateur | Prochaine release | Liste et journaux dans `kesh-gate-logs/rel…/` ; aucune phrase sans preuve |
| A5 | **Recette Claude Code C1-C3** sur une installation jetable — puis retirer les huit « non éprouvé » ; **ouvrir l'issue qui la trace** (A3 de la rétrospective 32, non faite) | Guy (recette), orchestrateur (issue, textes) | Avant toute activation du MCP sur les livres réels | Issue ouverte ; fermée par la PR qui retire les marques |
| A6 | **Verser les registres de l'Epic 32 et aligner les neuf *Status* `32-*`**, ou décider de ne plus le faire | Orchestrateur, sur décision de Guy (Q7) | Prochaine story | `grep -cE '^#{2,4} C-32-' epic-32-choix-autonomes.md` = 618 et les *Status* à `done`, ou décision écrite |
| A7 | **Tracer les constats de cette rétrospective** (« on trace toujours ») : #896 ou #897 à fermer comme doublon ; la date de la 0.17.1 (CHANGELOG et macro 2026-11-03 pour une publication du 2026-11-04, P4) ; l'absence de règle sur le label des issues de LOW (`bug` ou `technical-debt`, Q2) | Orchestrateur (cette rétrospective ne peut pas écrire sur GitHub) | Avec la PR de cette rétrospective | Doublon fermé ; issue de la date ouverte ou CHANGELOG corrigé |
| A8 | **Nettoyer** les 32 `/tmp/kesh-e2e-*` et rendre la règle de permission active pour les agents (reprise de A8 de la rétrospective 32) | Guy | À son retour | `ls -d /tmp/kesh-e2e-*` vide ; un agent supprime son dossier sans refus |
| A9 | **Mettre à jour `serveur-mcp-decisions.md`** : la 0.17 n'a porté ni les propositions ni #235 ; l'échéance de ces deux décisions est à refixer (Q6) | Orchestrateur, sur réponse de Guy | Après Q6 | La note dit à quelle release vont les propositions et #235 |

---

# Partie 2 — La suite

## 9. Prochaine étape : il n'y a pas d'epic suivant défini

**Aucun epic n'est ouvert ni planifié** : pas de clé `epic-33` au registre, pas de fichier d'epic dans
`planning-artifacts/`, et l'Epic 33 annoncé par la rétrospective 32 n'a jamais eu de kickoff. Ce qui suit est une liste
de candidats, **sans ordre décidé** — l'ordre est à Guy (§ 10, Q6).

- **#890** (P1 sécurité) — le seul P1 neuf, et le plus court ; la durée de session portée à 60 minutes allonge sa
  fenêtre (§ 2.4).
- **#847** (P2) — Kesh fait lui-même la sauvegarde complète avant une migration. C'est l'erreur du 2026-10-31 que Guy a
  nommée ; tant qu'elle est ouverte, chaque release à migration repose sur la mémoire de l'administrateur, et la 0.17.3
  en est une (Q11).
- **La pile de bugs** (87, dont 14 P2 et 25 nés de la série) : une story de défauts qui solde les LOW mécaniques des
  quinze issues de LOW ouvertes et une sélection de P2 — **#817** (pays contrôlé à la forme seulement), **#819** (entités
  CAMT ignorées ; décision de fond, elle change la clé de doublon des relevés déjà importés), #729, #742, #755, #776 —,
  avec #803, #804, #805 (A6 de la rétrospective 32). C'est le seul levier sur l'engagement 1.
- **#868** est faite (fermée par la 0.17.2, sous la forme décidée par Guy : une annulée repasse en brouillon, et le
  brouillon se supprime) ; #887 (factures clientes annulées, P4) en est le pendant côté client.
- **#885** est faite (0.17.2) ; **#886** (retrouver la date et l'échéance dans le document importé) en est la suite.
- **#895** — avertir avant la fermeture de session, « Rester connecté » (demande de Guy du 2026-11-04).
- **CR #877** — facturer aux locataires les acomptes et le décompte des charges (énergie, eau, index de compteurs) :
  une fonction neuve, de la taille d'un epic.
- **Les propositions de l'agent et #235** (P1, art. 957a CO) — le contenu prévu de la 0.17, reporté (§ 1.1).
- **La 0.18 (extraction fiscale vaudoise)** — pas avant le début 2027 selon les décisions du 2026-10-24 ; elle exige les
  pièces jointes sur les écritures déductibles, donc **#235 avant elle**.
- **Les dépendances** : #790 (zip), #791 (printpdf/lopdf), #252.

**Décompte de départ** : 239 issues ouvertes, 87 bugs, 14 P1 au tag v0.17.3.

---

## 10. Questions pour Guy

1. **L'objectif d'issues du prochain tag.** À la lettre de la règle, c'est « moins de 239 ». Il a été manqué quatre fois
   de suite, d'un écart qui se resserre (+48, +3, +2, +1). *Recommandation* : garder « moins de 239 », et le rendre
   atteignable en mettant en tête une story de défauts qui solde des LOW mécaniques — c'est ce qui a fait baisser le
   compte à la 32-1.
2. **La cible de bugs, et le label des issues de LOW.** L'engagement 1 n'est pas tenu depuis quatre périodes
   (49 → 49 → 51 → 66 → 87). *Recommandation* : une cible chiffrée modeste au prochain tag — **moins de 87 bugs** — ; et
   une règle écrite : une issue qui regroupe des LOW de revue porte `technical-debt`, non `bug`, sauf si l'un des LOW
   est un comportement faux (aujourd'hui #875, #883, #892, #893 portent `bug`). Cela ne change aucun défaut, mais rend
   le compte des bugs lisible.
3. **#890 en tête de la prochaine release ?** C'est un P1 de sécurité, ouvert à la 0.17.2, dont seul le texte a été
   corrigé ; la session à 60 minutes allonge la fenêtre d'un jeton lu dans une copie de la base. *Recommandation* : oui,
   en première story, avec l'invalidation des jetons existants à la mise à jour (une reconnexion unique).
4. **Le verrou commun de station** (`/tmp/station-heavy.lock`, convenu avec la session mybibli le 2026-11-05). Faut-il
   l'inscrire au `CLAUDE.md` (§ *Plafonds mémoire*) et à `docs/agents-orchestration.md`, et peut-être dans l'outil
   `mem-guard` lui-même ? *Recommandation* : oui, dans les trois — l'outil global est le seul endroit que **toutes** les
   sessions, de tous les projets, traversent ; une règle écrite dans le seul dépôt de kesh ne lie pas mybibli.
5. **La règle « une release corrective quand un bug gêne la production »**, appliquée trois fois sans être écrite.
   *Recommandation* : l'écrire au `CLAUDE.md`, avec ses bornes — sans migration si possible, un défaut qui gêne la tenue
   des livres, la sauvegarde rappelée en tête des notes — ; et dire si une corrective peut embarquer d'autres bugs
   « faciles » (ce qui a été fait aux 0.17.1 et 0.17.2, sur ta demande).
6. **Qu'est-ce qui vient ensuite, et que deviennent les décisions du 2026-10-24 ?** Les propositions de l'agent et #235
   étaient la 0.17.0 ; la série a suivi la production. *Recommandation* : une release courte d'abord (#890, une story
   de défauts, et #847 si tu veux qu'elle précède la prochaine migration), puis un epic qui porte #235 — un P1 légal,
   préalable à la 0.18 — ; les propositions de l'agent après la recette Claude Code (A5).
7. **Les registres de l'Epic 32** (A2 de la rétrospective 32, non fait depuis deux rétrospectives) : les verser, ou y
   renoncer ? *Recommandation* : y renoncer par écrit — la série 0.17 tient ses registres par release, sans registre
   d'epic, et personne ne les a relus ensemble ; aligner seulement les neuf *Status* `32-*`, qui sont faux.
8. **Le conteneur arrêté `kesh-rel0173-boot-0172`** (test de refus de la 0.17.2, `Exited (137)`) : le supprimer ?
   *Recommandation* : oui ; il ne sert plus, ses journaux sont dans `kesh-gate-logs/rel0173/`. Le dump et le
   `.keshbackup` de démonstration du même dossier (`boot-0172-dump.sql`, `boot-0172.keshbackup`) peuvent partir avec lui.
9. **La clôture d'une validation par ta décision** (v016-1, close à la sixième passe sur un MEDIUM). *Recommandation* :
   l'admettre par écrit comme troisième sortie de boucle, à côté de la passe sans MEDIUM et du plafond — « le Project
   Lead clôt, le MEDIUM est renvoyé à l'étape 0 et déclaré au Change Log » —, pour qu'elle ne se réinvente pas.
10. **Les décomptes de tête du `CLAUDE.md`** disent « 241 issues ouvertes à la préparation du tag, dont 88 bugs » et « 239
    attendues » : le tag v0.17.3 porte **239 et 87**. Et le paragraphe *Kesh est en production* dit que le NAS « tourne
    en 0.17.1 ». *Recommandation* : que l'orchestrateur annote ces deux lignes (le compte au tag ; la version en service,
    que toi seul connais) — cette rétrospective n'écrit pas au `CLAUDE.md`.
11. **La sauvegarde avant la mise à jour du NAS en 0.17.3.** La 0.17.3 porte une migration : la 0.17.2 (et la 0.17.1)
    refusent ensuite de démarrer, et le retour passe **uniquement** par la sauvegarde d'avant — base, `.keshbackup`
    **et** dossier des documents. #847 est ouverte : Kesh ne la prend pas de lui-même. *Recommandation* : la faire avant
    de changer l'image, et nous dire qu'elle est faite ; la mise à jour de la 0.17.0 n'a pas laissé de trace de
    sauvegarde au journal.
12. **La recette Claude Code C1-C3** (A5) : quand, et sur quelle installation jetable ? *Recommandation* : avant toute
    activation du MCP sur les livres réels du NAS, avec la vérification de DSM et de `X-Forwarded-For` par le journal du
    filtre (A7 de la rétrospective 32) dans le même geste.

[#235]: https://github.com/guycorbaz/kesh/issues/235
[#789]: https://github.com/guycorbaz/kesh/issues/789
[#847]: https://github.com/guycorbaz/kesh/issues/847
[#876]: https://github.com/guycorbaz/kesh/issues/876
[#890]: https://github.com/guycorbaz/kesh/issues/890
[#891]: https://github.com/guycorbaz/kesh/issues/891
