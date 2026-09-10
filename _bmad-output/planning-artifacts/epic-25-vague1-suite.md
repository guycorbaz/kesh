# Epic 25 — Vague 1 (suite) : la piste de contrôle et les annulations

**Kickoff : 2026-09-10.** Cet epic **termine la vague 1** du plan d'action de l'audit
(`audit-experts-2026-08-26.md`). L'Epic 24 en a livré le quart — le défaut fondateur du cycle
d'encaissement, le grand livre, la contre-passation, le gel, le verrou de période et les comptes
de clôture. **Seize issues restent ouvertes au jalon GitHub « Vague 1 ».**

⚠️ **L'Epic 24 s'appelait « vague 1 » et la vague n'était pas finie.** La rétrospective du
2026-09-10 l'a constaté, et le README a été rectifié le même jour. Cet epic-ci ne reprend pas
l'ambiguïté : il ne se clôt qu'avec **le jalon à zéro issue ouverte**.

⚠️ **Contrepartie BMAD du jalon GitHub « Vague 1 »** — les issues tiennent le détail et le
décompte, cet epic tient l'ordre et le raisonnement.

## Ce qui reste, et pourquoi c'est dans cet ordre

Le critère de tri est celui de l'audit : **ce qui rend les livres non opposables passe avant ce
qui les rend incommodes.**

### 25-1 — La piste de contrôle : la rendre inaltérable, puis lisible

**Issues : [#376], [#377], [#379], [#378].** C'est le III.3 de l'audit, et **c'est la
contrepartie directe de ce que l'Epic 24 vient de livrer** : la contre-passation et le gel
rendent les corrections *apparentes*, mais la seule trace de qui a corrigé quoi est effaçable
par deux chemins et lisible par aucun.

Le module déclare *« Pas de méthode delete : CO art. 957-964 impose la conservation 10 ans. Les
entrées sont inamovibles. »* — **la phrase n'est pas tenue** :

- `audit_log` figure dans `TABLES_TO_TRUNCATE` : l'import d'un `.keshbackup` **remplace
  intégralement** la piste ([#376]) ;
- ⚠️ `reset_demo` exécute un `DELETE FROM audit_log` **non scopé**, et sa route est montée dans
  le bloc « tout rôle authentifié », sans `require_admin_role` — *sa seule protection est un
  état, pas un droit* ([#377]) ;
- ni la gestion des utilisateurs ni la modification de la société ne sont tracées : **un
  changement de rôle vers Comptable ne laisse aucune trace** ([#379]) ;
- ⚠️ **aucune route, aucun écran ne permet de la consulter** ([#378]).

⛔ **L'ordre à l'intérieur de la story n'est pas indifférent** : rendre la piste *lisible* avant
de la rendre *inaltérable* publierait une promesse qu'un `reset_demo` dément. Fermer les deux
chemins d'effacement d'abord.

*« Apparent » et « archivé dans une table que personne ne peut lire » ne sont pas la même
chose.*

### 25-2 — Les gardes structurelles : numérotation et type de compte

**Issues : [#381], [#382].** Deux gardes absentes qui rendent les livres contestables.

- **La numérotation des écritures admet des trous et la réutilisation d'un numéro** ([#381]).
  Une séquence comptable qui saute ou se répète ne prouve plus l'exhaustivité.
- **Le type d'un compte mouvementé est modifiable sans garde** ([#382], III.6) — *« la clôture
  est contournable »* : retyper un compte après coup déplace ses soldes d'un état à l'autre sans
  qu'aucune écriture ne l'explique. ⚠️ Voisin de [#274] (KF ouverte : retyper reclassifie
  silencieusement).

### 25-3 — Défaire ce que l'Epic 24 a permis de faire

**Issues : [#414], [#418].** L'Epic 24 a livré l'encaissement et le rapprochement ; **il n'a pas
livré le moyen de les défaire**. Une erreur d'imputation sur un règlement ou un rapprochement
est aujourd'hui **incorrigible autrement qu'à la main**, et le gel de la 24-4b interdit
précisément la main.

⚠️ C'est le prix d'une story qui ferme une porte sans ouvrir la suivante : le gel est juste, et
il rend cette story-ci nécessaire.

### 25-4 — Propager le résiduel

**Issues : [#416], [#420].** La 24-2 a introduit le règlement partiel ; le résiduel n'est pas
propagé aux rapports agrégés — balance âgée et échéancier ([#416]) — et **le score de
rapprochement compare au TTC et non au résiduel**, si bien que le virement du solde d'une
facture partiellement réglée **score 0** ([#420]).

### 25-5 — Les états et l'export

**Issues : [#385], [#386].** La **Balance des comptes ne concorde pas avec le Bilan** — il
manque la colonne « solde d'ouverture » ([#385], III.4) ; et **l'export de souveraineté omet les
achats, les avoirs, les projets et la piste d'audit** ([#386], V.6) — *il promet plus qu'il ne
tient*.

### 25-6 — Le tableau de bord, et deux dettes isolées

**Issues : [#388], [#389], [#384], [#387].**

- La tuile **« Dernières écritures » n'appelle rien** et affiche toujours « Aucune écriture »
  ([#388]) — *une tuile qui est un décor*.
- Le **« solde bancaire » est le solde comptable**, donc faux ([#389]).
- **Imputer l'écart d'un règlement partiel** — escompte, frais bancaires, perte sur débiteur
  ([#384]).
- **Le PDF d'une facture n'est jamais archivé**, il est régénéré à la volée ([#387], IV.9) —
  un document réémis n'est pas le document envoyé.

## Ce qui est hors de cet epic

- **La TVA** — vague 2, douze issues. ⚠️ **Différée, non éteinte** : le fiscaliste la classait
  en tête *pour un assujetti*. Tenable tant que Kesh ne tient pas les livres d'un assujetti.
- **La comptabilité personnelle** — vague 3, sept issues, et **le chantier suivant** par
  arbitrage du Project Lead du 2026-09-10.
- **[#427] et [#429]** — les flux de réconciliation et les réglages de facturation qui ne
  vérifient pas `postable` côté serveur. Même famille : écran qui filtre, serveur muet. Ni l'un
  ni l'autre ne met les comptes de clôture en cause.

## Priorité de cible

Inchangée depuis le 2026-08-26 : **1. comptabilité personnelle — 2. indépendant — 3. PME.**
Cet epic sert les trois : une piste de contrôle inaltérable et des livres qui concordent ne
sont pas un besoin de profil, c'est le socle.

## Ce qui clôt cet epic

**Le jalon GitHub « Vague 1 » à zéro issue ouverte** — et rien d'autre. Pas « les stories sont
`done` » : c'est exactement la confusion que l'Epic 24 a laissée derrière lui.
