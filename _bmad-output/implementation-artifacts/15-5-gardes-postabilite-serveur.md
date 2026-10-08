# Story 15.5 : Gardes de postabilité côté serveur — réconciliation, règles, réglages de facturation, compte bancaire

## Status

split

⛔ **CORPS VIDÉ — cette fiche ne contient plus ni critères, ni tâches, ni inventaire.** Elle ne garde
que les pointeurs vers ses deux moitiés et l'historique de la passe qui a conduit au découpage.
*(La définition du statut `split` l'impose ; précédents : 15-1, 17-2.)* La version complète d'avant
découpage se lit au commit `428985c8`.

## Les deux sous-stories

| | fiche | ce qu'elle porte | issues |
|---|---|---|---|
| **15-5a** | `15-5a-refus-non-imputable.md` | **Socle** : la variante `DbError::AccountsNotPostable`, le code `ACCOUNT_NOT_POSTABLE`, son message (4 locales, pluriel), sa conversion sur la saisie manuelle, l'écriture d'ouverture et les trois gardes de la 24-5 ; les tests qui figeaient l'ancien code ; l'ordre des causes ; le détail `accountNumbers` | `refs #427`, `refs #429` |
| **15-5b** | `15-5b-gardes-surfaces-neuves.md` | **Rollout** : les gardes neuves — rapprochement manuel et ventilé, acceptation `split` et `rule`, `get_proposals`, création / modification / réactivation des règles, six réglages de facturation, compte comptable d'un compte bancaire ; l'écran des refus par lot ; le manuel | `closes #427`, `closes #429`, `closes #492`, `closes #519` |

⚠️ **L'ordre n'est pas indifférent** : la 15-5b émet la variante que la 15-5a pose. Elle ne commence
qu'après le merge de la 15-5a.

## Pourquoi le découpage

La passe de validation P1 (trois lentilles Sonnet) n'a rien trouvé au-dessus de MEDIUM, mais deux
lentilles (A : M5, C : C-4) ont relevé que la story franchissait le **critère de périmètre** de la
§ *Règle de splitting préventif* — `kesh-db`, `kesh-api` (erreurs + cinq modules de routes),
`kesh-i18n`, `frontend`, manuels, CHANGELOG — et que la fiche s'en dispensait par l'argument « les
gardes sont mécaniques », qui n'est pas une dérogation prévue (la seule codifiée : les cycles de
dépendance Cargo). L'orchestrateur a découpé selon le patron que la règle prescrit — **story-zéro qui
pose le patron, puis rollout** (choix **C7** de `epic-15-choix-autonomes.md`). Les remédiations de la
passe P1 ont été appliquées **dans les deux fiches filles**, pas ici.

## Décisions prises pour la story et où elles vivent

| choix | objet | fiche |
|---|---|---|
| C3 | forme du refus : variante dédiée, code `ACCOUNT_NOT_POSTABLE` | 15-5a (pose), 15-5b (emploie) |
| C4 | exemption « inchangé » : réglages, compte bancaire, PATCH de règle | 15-5b |
| C5 | compte d'un compte bancaire : postabilité seule ; le reste de #474 à la 15-6 | 15-5b |
| C6 | angles morts assumés (fiche article, compte bancaire à l'usage, rôles, avoir) | 15-5b |
| C7 | découpage | les deux |
| C8 | libellés traduits pour tous les codes de `failed[]` (#492) | 15-5b |
| C9 | règle périmée : plus proposée, réactivation refusée, pas de migration | 15-5b |
| C10 | contrôle « inchangé » dans la transaction, ordre des erreurs conservé | 15-5b |
| C11 | réécriture du § *Règles d'affectation automatique* (#519) | 15-5b |
| C12 | `withCurrentAccount` sur les `<select>` du compte bancaire | 15-5b |
| C13 | clé de détail `accountNumbers` commune ; ordre des causes | 15-5a, 15-5b |
| C14 | les `catch` de l'écran des règles lisent `ApiError` (« [object Object] ») | 15-5b |

## Change Log

- 2026-10-08 — Spécification initiale (bmad-create-story, en autonomie), commit `428985c8`. Statut
  `ready-for-dev`. Choix C3–C6 consignés.
- 2026-10-08 — **Passe de validation P1** (prompt versionné `15-5-validate-prompt-p1.md` ; trois
  lentilles Sonnet en contexte frais : A auditeur d'acceptation, B chasseur de chemins non gardés,
  C régressions et bords). **0 CRITICAL, 0 HIGH.** Bruts : A 5 MEDIUM + 6 LOW, B 2 MEDIUM + 2 LOW, C 4
  MEDIUM + 4 LOW (11 MEDIUM, 12 LOW). Après fusion des doublons inter-lentilles : **7 MEDIUM et 8 LOW
  distincts** :

  | finding | sévérité | lentilles | objet | sort |
  |---|---|---|---|---|
  | M1 = C-1 | MEDIUM | A, C | grep de T2 ratant `invoice_settlement.rs:354` ; `supplier_invoices_repository.rs:424`, `:470` non nommés | 15-5a AC7, T0 |
  | M2 = B1 | MEDIUM | A, B | « Aucun changement d'écran requis » faux : `failed[]` en code brut | 15-5b AC14 (C8, #492) |
  | M3 ≈ C-2 | MEDIUM | A, C | PATCH `active:true` ressuscitant une règle périmée ; état visible non dit | 15-5b AC8 (b), AC9, AC17 (C9) |
  | M4 | MEDIUM | A | ordre « non imputable ET mauvais type » non tranché | 15-5a AC4 |
  | M5 = C-4 | MEDIUM | A, C | règle de splitting franchie | **découpage (C7)** |
  | B2 (+ C-6 LOW) | MEDIUM | B, C | lecture de la valeur en place avant validation : ordre des erreurs changé, hors transaction | 15-5b AC8, AC12 (C10) |
  | C-3 | MEDIUM | C | `## [Unreleased]` inexistant au CHANGELOG | 15-5a AC9, 15-5b AC18 |
  | L1 | LOW | A | chemins frontend incomplets | 15-5b |
  | L2 + B4 | LOW | A, B | détail structuré du 400 et de `failed[]` non fixé | `accountNumbers` (C13) |
  | L3 | LOW | A | pluriel et apostrophe du message | 15-5a AC2 |
  | L4 | LOW | A | priorité 404 > 400 de `post_split` et messages des 4 locales non testés | 15-5a T2, 15-5b AC16 |
  | L5 + C-8 | LOW | A, C | manuel : § *Règles d'affectation* décrit un écran inexistant ; `admin-manual` sans objet | 15-5b AC17 (C11, #519) |
  | L6 | LOW | A | inventaire (a) à ±1 | 15-5b, recompté |
  | B3 + C-5 | LOW | B, C | numéros de ligne (`fn` vs appel ; dépôt vs route homonyme ; `:1656`) | 15-5b |
  | C-7 | LOW | C | comportement du `<select>` Svelte sur une valeur absente des options | 15-5b AC13 (C12), vérifié au code |

  **Trouvés pendant la remédiation** (en refaisant les axes manuel et écrans) : le § *Acceptation par
  lot* du manuel (`user-manual.tex:1532`) dit l'opération « atomique » et renvoie au `CLAUDE.md` →
  15-5b AC17 ; `RuleFormModal.svelte:104` et `RulesList.svelte:64`, `:87` affichent « [object Object] »
  pour un `ApiError` → 15-5b AC14 (C14). Le même motif hors module (`reports/+page.svelte:210`,
  `settings/+page.svelte:236`, `:260`) est signalé pour une issue.

  **Découpage (C7)** : statut `split`, corps vidé. Les deux fiches filles portent chacune la remédiation
  qui leur revient et leur propre Change Log ; la passe P2 se lance **sur chacune**.
