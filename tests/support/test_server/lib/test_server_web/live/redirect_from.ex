defmodule TestServerWeb.RedirectLive do
  use TestServerWeb, :live_view
  use TestServerNative, :live_view

  def mount(_params, _session, socket) do
    socket =
      case socket.assigns.live_action do
        :cross_session ->
          socket
          |> Phoenix.LiveView.redirect(to: ~p"/cross_session")

        _ ->
          socket
          |> Phoenix.LiveView.redirect(to: ~p"/redirect_to")
      end

    {:ok, socket}
  end

  def render(assigns) do
    ~H"""
    <p>
      you should have been redirected
    </p>
    """
  end
end

defmodule TestServerWeb.RedirectLive.Jetpack do
  use TestServerNative, [:render_component, format: :jetpack]

  def render(assigns, _) do
    ~LVN"""
    <Box size="fill" background="system-blue">
      <Text align="Center">
        Should Have Been Redirected
      </Text>
    </Box>
    """
  end
end

defmodule TestServerWeb.RedirectLive.SwiftUI do
  use TestServerNative, [:render_component, format: :swiftui]

  def render(assigns, _interface) do
    ~LVN"""
    <VStack>
      <Text>
        Should Have Been Redirected
      </Text>
    </VStack>
    """
  end
end
