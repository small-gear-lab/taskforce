mod import;
mod intake;

use crate::plugin::LogicalFieldLabel;

pub use import::import_chatwork_url;

pub fn logical_field_labels() -> &'static [LogicalFieldLabel] {
    &[
        LogicalFieldLabel {
            physical_path: "requester",
            msgid: "Requester",
        },
        LogicalFieldLabel {
            physical_path: "request_url",
            msgid: "This Request",
        },
        LogicalFieldLabel {
            physical_path: "related_request_url",
            msgid: "Related Request",
        },
        LogicalFieldLabel {
            physical_path: "summary",
            msgid: "Summary",
        },
        LogicalFieldLabel {
            physical_path: "abstract",
            msgid: "Abstract",
        },
        LogicalFieldLabel {
            physical_path: "description",
            msgid: "Description",
        },
        LogicalFieldLabel {
            physical_path: "production_rollout",
            msgid: "Production Rollout",
        },
        LogicalFieldLabel {
            physical_path: "template_kind",
            msgid: "Template Kind",
        },
        LogicalFieldLabel {
            physical_path: "target_sites",
            msgid: "Target Sites",
        },
        LogicalFieldLabel {
            physical_path: "target_sites[].label",
            msgid: "Site Label",
        },
        LogicalFieldLabel {
            physical_path: "target_sites[].site_code",
            msgid: "Site Code",
        },
        LogicalFieldLabel {
            physical_path: "target_sites[].raw",
            msgid: "Scope Line",
        },
        LogicalFieldLabel {
            physical_path: "chatwork",
            msgid: "Chatwork",
        },
        LogicalFieldLabel {
            physical_path: "chatwork.requester",
            msgid: "Requester",
        },
        LogicalFieldLabel {
            physical_path: "chatwork.request_url",
            msgid: "This Request",
        },
        LogicalFieldLabel {
            physical_path: "chatwork.related_request_url",
            msgid: "Related Request",
        },
        LogicalFieldLabel {
            physical_path: "chatwork.summary",
            msgid: "Summary",
        },
        LogicalFieldLabel {
            physical_path: "chatwork.abstract",
            msgid: "Abstract",
        },
        LogicalFieldLabel {
            physical_path: "chatwork.description",
            msgid: "Description",
        },
        LogicalFieldLabel {
            physical_path: "chatwork.production_rollout",
            msgid: "Production Rollout",
        },
        LogicalFieldLabel {
            physical_path: "chatwork.template_kind",
            msgid: "Template Kind",
        },
        LogicalFieldLabel {
            physical_path: "chatwork.target_sites",
            msgid: "Target Sites",
        },
        LogicalFieldLabel {
            physical_path: "chatwork.target_sites[].label",
            msgid: "Site Label",
        },
        LogicalFieldLabel {
            physical_path: "chatwork.target_sites[].site_code",
            msgid: "Site Code",
        },
        LogicalFieldLabel {
            physical_path: "chatwork.target_sites[].raw",
            msgid: "Scope Line",
        },
        LogicalFieldLabel {
            physical_path: "chatwork.source",
            msgid: "Message Source",
        },
        LogicalFieldLabel {
            physical_path: "chatwork.source.kind",
            msgid: "Kind",
        },
        LogicalFieldLabel {
            physical_path: "chatwork.source.room_id",
            msgid: "Room ID",
        },
        LogicalFieldLabel {
            physical_path: "chatwork.source.message_id",
            msgid: "Message ID",
        },
        LogicalFieldLabel {
            physical_path: "chatwork.source.url",
            msgid: "Message URL",
        },
        LogicalFieldLabel {
            physical_path: "chatwork.source.sender",
            msgid: "Sender",
        },
        LogicalFieldLabel {
            physical_path: "chatwork.source.sender.account_id",
            msgid: "Sender Account ID",
        },
        LogicalFieldLabel {
            physical_path: "chatwork.source.sender.name",
            msgid: "Sender Name",
        },
        LogicalFieldLabel {
            physical_path: "chatwork.source.sent_at",
            msgid: "Sent At",
        },
        LogicalFieldLabel {
            physical_path: "chatwork.source.body_raw",
            msgid: "Body",
        },
    ]
}
