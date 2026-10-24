// Story 31-1c (#503, AC31) — l'état de la sauvegarde automatique, partagé entre
// le panneau d'administration et le bandeau du layout : une action du panneau
// (enregistrer, « Sauvegarder maintenant ») le met à jour, et le bandeau suit
// sans attendre son prochain sondage (R9 de la validation P5).
//
// Pattern objet avec getter (l'export direct d'un `$state` n'est pas
// réassignable depuis un importeur — cf. `auth.svelte.ts`).

import { fetchBackupSchedule, type BackupSchedule } from './backup-schedule.api';

let _value = $state<BackupSchedule | null>(null);

export const backupScheduleState = {
	/** Le dernier état connu ; `null` tant qu'il n'est pas lu, ou si la lecture a échoué. */
	get value(): BackupSchedule | null {
		return _value;
	},
};

/** Pose l'état rendu par une action du panneau. */
export function setBackupScheduleState(value: BackupSchedule | null): void {
	_value = value;
}

/**
 * Relit l'état. Un échec de lecture remet l'état à `null` — le bandeau ne
 * s'affiche alors pas (AC31 : un échec de cette lecture n'affiche aucun
 * bandeau et ne casse pas le layout, pas de toast).
 */
export async function refreshBackupScheduleState(): Promise<void> {
	try {
		_value = await fetchBackupSchedule();
	} catch {
		_value = null;
	}
}
