function urlBase64ToUint8Array(base64String) {
  const padding = "=".repeat((4 - (base64String.length % 4)) % 4);
  const base64 = (base64String + padding)
    .replace(/\-/g, "+")
    .replace(/_/g, "/");

  const rawData = window.atob(base64);
  const outputArray = new Uint8Array(rawData.length);

  for (let i = 0; i < rawData.length; ++i) {
    outputArray[i] = rawData.charCodeAt(i);
  }
  return outputArray;
}

document.addEventListener("alpine:init", () => {
  Alpine.data("pushNotifications", () => ({
    async unsubscribe() {
      if (!("serviceWorker" in navigator)) return;

      try {
        const registration = await navigator.serviceWorker.getRegistration();
        if (!registration) return;

        const subscription = await registration.pushManager.getSubscription();
        if (subscription) {
          // Unsubscribe from push service
          await subscription.unsubscribe();

          // server-side unsubscription is handled on the server-side
          // (detect and delete invalid subscription on next push)

          this.subscribed = false;
        }
      } catch (err) {
        console.error("Error unsubscribing:", err);
      }
    },

    async subscribe() {
      if (!("serviceWorker" in navigator) || !("PushManager" in window)) {
        alert("Push notifications not supported");
        return;
      }

      try {
        const registration = await navigator.serviceWorker.register("/sw.js");
        const publicKeyRes = await fetch("/notification/push/public-key");
        const publicKey = await publicKeyRes.text();
        const subscription = await registration.pushManager.subscribe({
          userVisibleOnly: true,
          applicationServerKey: urlBase64ToUint8Array(publicKey),
        });

        await fetch("/notification/push/subscribe", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(subscription),
        });

        this.subscribed = true;
      } catch (err) {
        console.error("Error subscribing to push notifications:", err);
      }
    },
    subscribed: false,
    init() {
      // Check subscription status on load
      this.checkSubscription();
    },
    async checkSubscription() {
      console.log("checkSubscription");
      if (!("serviceWorker" in navigator)) return;

      const registration = await navigator.serviceWorker.getRegistration();
      console.log("registration: ", registration);
      if (registration) {
        const subscription = await registration.pushManager.getSubscription();
        this.subscribed = !!subscription;
      }
    },
  }));
});
