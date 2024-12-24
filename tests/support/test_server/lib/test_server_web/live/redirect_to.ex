defmodule TestServerWeb.RedirectToLive do
  use TestServerWeb, :live_view
  use TestServerNative, :live_view

  def mount(_params, session, socket) do
    socket =
      socket
      |> assign(:action, inspect(socket.assigns.live_action || :default))

    {:ok, socket}
  end

  def render(assigns) do
    ~H"""
    <p>
      Redirect to <%= assigns.action%>
    </p>
    """
  end
end

defmodule TestServerWeb.RedirectToLive.Jetpack do
  use TestServerNative, [:render_component, format: :jetpack]

  def render(assigns, _) do
    ~LVN"""
    <Box size="fill" background="system-blue">
      <Text align="Center">
        Redirected to <%= assigns.action %>!
      </Text>
    </Box>
    """
  end
end

defmodule TestServerWeb.RedirectToLive.SwiftUI do
  use TestServerNative, [:render_component, format: :swiftui]

  def render(assigns, _interface) do
    ~LVN"""
    <VStack>
      <Text>
        Redirected to <%= assigns.action %>!
      </Text>
    </VStack>
    """
  end
end
