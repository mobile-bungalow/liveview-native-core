defmodule TestServerWeb.UserRegistrationLive do
  use TestServerWeb, :live_view
  use TestServerNative, :live_view

  def mount(_params, _session, socket) do
    socket =
      socket
      |> assign(trigger_submit: false, check_errors: false)

    {:ok, socket, temporary_assigns: [form: nil]}
  end

  def handle_event("save", %{"user" => user_params}, socket) do
    {:ok, socket}
  end

  def handle_event(_e, _p, socket) do
    {:noreply, socket}
  end

  def render(assigns) do
    ~H"""
    <div class="mx-auto max-w-sm">
      <.header class="text-center">
        Register for an account
        <:subtitle>
          Already registered?
          <.link navigate={~p"/users/log_in"} class="font-semibold text-brand hover:underline">
            Sign in
          </.link>
          to your account now.
        </:subtitle>
      </.header>

      <.simple_form
        for={@form}
        id="registration_form"
        phx-submit="save"
        phx-change="validate"
        phx-trigger-action={@trigger_submit}
        action={~p"/users/log_in?_action=registered"}
        method="post"
      >
        <.error :if={@check_errors}>
          Oops, something went wrong! Please check the errors below.
        </.error>

        <.input value="" name="email" field={@form[:email]} type="email" label="Email" required />
        <.input  value="" name="password" field={@form[:password]} type="password" label="Password" required />

        <:actions>
          <.button phx-disable-with="Creating account..." class="w-full">Create an account</.button>
        </:actions>
      </.simple_form>
    </div>
    """
  end
end

defmodule TestServerWeb.UserRegistrationLive.SwiftUI do
  use TestServerNative, [:render_component, format: :swiftui]

  def render(assigns, _) do
    ~LVN"""
    <.simple_form
      for={@form}
      id="registration_form"
      phx-submit="save"
      phx-change="validate"
      phx-trigger-action={@trigger_submit}
      action={~p"/users/log_in?_action=registered"}
      method="post"
    >
      <.error :if={@check_errors}>
        Oops, something went wrong! Please check the errors below.
      </.error>

      <.input value="" name="email" field={@form[:email]} type="TextField" label="Email" class="keyboardType(.emailAddress)" autocomplete="off" />
      <.input  value="" name="password" field={@form[:password]} type="SecureField" label="Password" />

      <:actions>
        <.button type="submit">
          <Label>
            <Text template="title">Create an account</Text>
            <.image url={~p"/images/logo.png"} template="icon">
              <:empty></:empty>
              <:success class="rendering-mode-template resizable scaled-to-fit fg-white" />
              <:failure></:failure>
            </.image>
          </Label>
        </.button>
      </:actions>
    </.simple_form>
    """
  end
end
