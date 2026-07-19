/**
 * Re-exports publics de la feature `reminders` (envoi des rappels — Story 21-6b, #231).
 */

export type {
	ReminderCandidate,
	ContactGroup,
	ReminderListResponse,
	ReminderPreviewResponse,
	SendReminderRequest,
	ReminderResponse,
	AcceptedReminder,
	FailedReminder,
	SendReminderBatchResponse,
	ManualReminderRequest,
	DunningPauseResponse,
	PauseDunningRequest,
	ResumeDunningRequest,
} from './reminders.types';
export {
	listReminders,
	getReminderPreview,
	sendReminder,
	sendReminderBatch,
	recordManualReminder,
	listReminderHistory,
	pauseDunning,
	resumeDunning,
} from './reminders.api';
