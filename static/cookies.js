const userIdInput = document.getElementById("user_id");

if (Cookies.get("uuid")) {
  userIdInput.setAttribute("value", Cookies.get("uuid"));
} else {
  const newUUID = crypto.randomUUID();
  Cookies.set("uuid", newUUID);
  userIdInput.setAttribute("value", newUUID);
}
