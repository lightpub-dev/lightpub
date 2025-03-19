self.addEventListener("push", (event) => {
  const data = event.data.json();
  self.registration.showNotification(data.title, {
    body: data.body,
    data: { url: data.url },
  });
});

self.addEventListener("notificationclick", (event) => {
  event.notification.close();
  clients.openWindow(event.notification.data.url);
});
