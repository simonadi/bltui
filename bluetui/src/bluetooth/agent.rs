use std::{
    fmt::{Debug, Display, Formatter},
    sync::Arc,
    time::Duration,
};

use dbus::{
    blocking::BlockingSender,
    channel::MatchingReceiver,
    message::MatchRule,
    nonblock::{
        stdintf::org_freedesktop_dbus::RequestNameReply, NonblockReply, Proxy, SyncConnection,
    },
    Message,
};
use dbus_crossroads::{Crossroads, IfaceBuilder};

use log::{debug, info};

#[derive(Debug)]
pub enum AgentCapability {
    DisplayOnly,
    DisplayYesNo,
    KeyboardDisplay,
    KeyboardOnly,
    NoInputNoOutput,
}

impl Display for AgentCapability {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

pub struct Agent<'a> {
    path: dbus::Path<'a>,
    capability: AgentCapability,
    connection: Arc<dbus::nonblock::SyncConnection>,
}

impl Agent<'static> {
    pub fn new(path: &str, capability: AgentCapability) -> Agent<'static> {
        let (resource, connection) = dbus_tokio::connection::new_system_sync().unwrap();

        let _handle = tokio::spawn(async {
            let err = resource.await;
            panic!("Lost connection to D-Bus: {}", err);
        });

        Agent {
            path: dbus::Path::new(path).unwrap(),
            capability: capability,
            connection,
        }
    }

    pub async fn register_and_request_default_agent(&self) {
        let connection = std::sync::Arc::clone(&self.connection);

        let proxy = Proxy::new(
            "org.bluez",
            "/org/bluez",
            Duration::from_secs(2),
            connection,
        );
        let (): () = proxy
            .method_call(
                "org.bluez.AgentManager1",
                "RegisterAgent",
                (&self.path, &self.capability.to_string()),
            )
            .await
            .unwrap();

        info!("Registered the agent");

        let (): () = proxy
            .method_call(
                "org.bluez.AgentManager1",
                "RequestDefaultAgent",
                (&self.path,),
            )
            .await
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
        let c = std::sync::Arc::clone(&self.connection);

        // Spawn a task that polls the Dbus to check that the connection is still alive.
        // Panics when it's lost

        // self.request_name(&c).await.unwrap();

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
                |mut ctx, _cr, (device,): (dbus::Path,)| {
                    info!("Reiceved RequestPinCode command");
                    async move { ctx.reply(Ok(("pincode",))) }
                },
            );

            b.method_with_cr_async(
                "DisplayPinCode",
                ("device", "pincode"),
                (),
                |mut ctx, _cr, (device, pincode): (dbus::Path, String)| {
                    info!("Reiceved DisplayPinCode command");
                    async move { ctx.reply(Ok(())) }
                },
            );

            b.method_with_cr_async(
                "RequestPasskey",
                ("device",),
                ("passkey",),
                |mut ctx, _cr, (device,): (dbus::Path,)| {
                    info!("Reiceved RequestPasskey command");
                    async move { ctx.reply(Ok((1_u32,))) }
                },
            );

            b.method_with_cr_async(
                "DisplayPasskey",
                ("device", "passkey", "entered"),
                (),
                |mut ctx, _cr, (device, passkey, entered): (dbus::Path, u32, u16)| {
                    info!("Reiceved DisplayPasskey command");
                    async move { ctx.reply(Ok(())) }
                },
            );

            b.method_with_cr_async(
                "RequestConfirmation",
                ("device", "passkey"),
                (),
                |mut ctx, _cr, (device, passkey): (dbus::Path, u32)| {
                    debug!(
                        "Received RequestConfirmation command for device {} with passkey {}",
                        device, passkey
                    );
                    async move { ctx.reply(Ok(())) }
                },
            );

            b.method_with_cr_async(
                "RequestAuthorization",
                ("device",),
                (),
                |mut ctx, _cr, (device,): (dbus::Path,)| {
                    info!("Reiceved RequestAuthorization command");
                    async move { ctx.reply(Ok(())) }
                },
            );

            b.method_with_cr_async(
                "AuthorizeService",
                ("device", "uuid"),
                (),
                |mut ctx, _cr, (device, uuid): (dbus::Path, String)| {
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
