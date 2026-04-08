--!strict

local ui_types = require("./types")
local bridge = require("./bridge")

type SpinnerProps = ui_types.SpinnerProps
type SkeletonProps = ui_types.SkeletonProps

local module = {}

function module.spinner(props: SpinnerProps?): ()
    bridge.current_ui().spinner(props)
end

function module.skeleton(props: SkeletonProps?): ()
    bridge.current_ui().skeleton(props)
end

return module
