defmodule TestServerWeb.CrossSession do
  use TestServerWeb, :live_view
  use TestServerNative, :live_view

  def mount(_params, _session, socket) do
    {:ok, socket}
  end

  def render(assigns) do
    ~H"""
    <p>
      Redirected Accross Session!
    </p>
    """
  end
end

defmodule TestServerWeb.CrossSession.Jetpack do
  use TestServerNative, [:render_component, format: :jetpack]

  def render(assigns, _) do
    ~LVN"""
    <Box size="fill" background="system-blue">
      <Text align="Center">
        Redirected Accross Session!
      </Text>
    </Box>
    """
  end
end

defmodule TestServerWeb.CrossSession.SwiftUI do
  use TestServerNative, [:render_component, format: :swiftui]

  def render(assigns, _interface) do
    ~LVN"""
    <VStack>
      <Text>
        Redirected Accross Session!
      </Text>
    </VStack>
    """
  end
end
