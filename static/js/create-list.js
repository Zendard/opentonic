const form = document.getElementById("form")
const form_data = new FormData(form)
form.addEventListener("submit", submitForm)

async function submitForm(_) {
  console.log(form_data)
  const res = await fetch("api/create-list", {
    method: "POST",
    body: JSON.stringify(Object.fromEntries(form_data)),
    headers: { "Content-Type": "application/json" }
  })
  console.log(res)
  // Redirect to index if success
  if (res.status == 200) {
    window.location.href = window.location.href.replace("create-list", "")
  }
}
