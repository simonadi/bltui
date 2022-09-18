use std::sync::Arc;

use dbus::channel::MatchingReceiver;
use dbus::message::MatchRule;
use dbus::nonblock::stdintf::org_freedesktop_dbus::RequestNameReply;
use dbus::Message;
use dbus::{blocking::BlockingSender, nonblock::SyncConnection};
use dbus_crossroads::{Crossroads, IfaceBuilder};

use log::info;

pub struct Agent<'a> {
    path: dbus::Path<'a>,
    capability: String,
    connection: dbus::blocking::Connection,
}

impl Agent<'static> {
    pub fn new(path: &str, capability: &str) -> Agent<'static> {
        let connection = dbus::blocking::Connection::new_system().unwrap();
        Agent {
            path: dbus::Path::new(path).unwrap(),
            capability: capability.to_string(),
            connection,
        }
    }

    pub async fn register_agent(&self) {
        let m = Message::new_method_call(
            "org.bluez",
            "/org/bluez",
            "org.bluez.AgentManager1",
            "RegisterAgent",
        )
        .unwrap()
        .append2(&self.path, &self.capability);
        let r = self
            .connection
            .send_with_reply_and_block(m, std::time::Duration::from_secs(2))
            .unwrap();

        info!("message : {:?}", r);

        info!("Registered the agent");
    }

    pub async fn request_default_agent(&self) {
        let m = Message::new_method_call(
            "org.bluez",
            "/org/bluez",
            "org.bluez.AgentManager1",
            "RequestDefaultAgent",
        )
        .unwrap()
        .append1(&self.path);
        let r = self
            .connection
            .send_with_reply_and_block(m, std::time::Duration::from_secs(2))
            .unwrap();

        info!("Agent is now the default agent");
    }

    async fn request_name(&self, c: &Arc<SyncConnection>) -> Result<(), dbus::Error> {
        let request_reply = c.request_name("bluetui.agent", false, true, true).await?;

        match request_reply {
            RequestNameReply::AlreadyOwner => {
                info!("already owner");
            }
            RequestNameReply::Exists => {
                info!("exists");
            }
            RequestNameReply::InQueue => {
                info!("in queue");
            }
            RequestNameReply::PrimaryOwner => {
                info!("primary owner");
            }
        }

        Ok(())
    }

    pub async fn start(&self) {
        let (resource, c) = dbus_tokio::connection::new_system_sync().unwrap();

        // Spawn a task that polls the Dbus to check that the connection is still alive.
        // Panics when it's lost
        let _handle = tokio::spawn(async {
            let err = resource.await;
            panic!("Lost connection to D-Bus: {}", err);
        });

        self.request_name(&c).await.unwrap();

        let mut cr = Crossroads::new();
        cr.set_async_support(Some((
            c.clone(),
            Box::new(|x| {
                tokio::spawn(x);
            }),
        )));

        let iface_token = cr.register("org.bluez.Agent1", |b: &mut IfaceBuilder<()>| {
            b.method_with_cr_async("Release", (), (), |mut ctx, device, _: ()| {
                info!("Reiceved Release command");
                async move { ctx.reply(Ok(())) }
            });

            b.method_with_cr_async(
                "RequestPinCode",
                ("device",),
                ("pincode",),
                |mut ctx, cr, (device,): (dbus::Path,)| {
                    info!("Reiceved RequestPinCode command");
                    async move { ctx.reply(Ok(("pincode",))) }
                },
            );

            b.method_with_cr_async(
                "DisplayPinCode",
                ("device", "pincode"),
                (),
                |mut ctx, cr, (device, pincode): (dbus::Path, String)| {
                    info!("Reiceved DisplayPinCode command");
                    async move { ctx.reply(Ok(())) }
                },
            );

            b.method_with_cr_async(
                "RequestPasskey",
                ("device",),
                ("passkey",),
                |mut ctx, cr, (device,): (dbus::Path,)| {
                    info!("Reiceved RequestPasskey command");
                    async move { ctx.reply(Ok((1 as u32,))) }
                },
            );

            b.method_with_cr_async(
                "DisplayPasskey",
                ("device", "passkey", "entered"),
                (),
                |mut ctx, cr, (device, passkey, entered): (dbus::Path, u32, u16)| {
                    info!("Reiceved DisplayPasskey command");
                    async move { ctx.reply(Ok(())) }
                },
            );

            b.method_with_cr_async(
                "RequestConfirmation",
                ("device", "passkey"),
                (),
                |mut ctx, cr, (device, passkey): (dbus::Path, u32)| {
                    info!("Reiceved RequestConfirmation command");
                    async move { ctx.reply(Ok(())) }
                },
            );

            b.method_with_cr_async(
                "RequestAuthorization",
                ("device",),
                (),
                |mut ctx, cr, (device,): (dbus::Path,)| {
                    info!("Reiceved RequestAuthorization command");
                    async move { ctx.reply(Ok(())) }
                },
            );

            b.method_with_cr_async(
                "AuthorizeService",
                ("device", "uuid"),
                (),
                |mut ctx, cr, (device, uuid): (dbus::Path, String)| {
                    info!("Reiceved AuthorizeService command");
                    async move { ctx.reply(Ok(())) }
                },
            );

            b.method_with_cr_async("Cancel", (), (), |mut ctx, cr, _: ()| {
                info!("Reiceved Cancel command");
                async move { ctx.reply(Ok(())) }
            });
        });

        let address = self.path.clone();

        cr.insert(address, &[iface_token], ());

        tokio::spawn(async move {
            c.start_receive(
                MatchRule::new_method_call(),
                Box::new(move |msg, conn| {
                    cr.handle_message(msg, conn).unwrap();
                    true
                }),
            );
            futures::future::pending::<()>().await;
            unreachable!();
        });
    }
}
