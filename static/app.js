document.addEventListener("DOMContentLoaded", () => {
  document.body.addEventListener("htmx:responseError", (event) => {
    console.warn("Readsmith htmx response error", event.detail);
  });
});
