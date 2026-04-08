--!strict

local ui_types = require("./types")
local bridge = require("./bridge")

type VirtualListProps = ui_types.VirtualListProps

local module = {}

function module.render(id: string, items: { string }, props: VirtualListProps?): (number, boolean)
    return bridge.current_ui().virtual_list(id, items, props)
end

return module
