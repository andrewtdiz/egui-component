--!strict

local module = {}

function module.merge(...: any): any
    local merged = {}
    for index = 1, select("#", ...) do
        local candidate = select(index, ...)
        if candidate ~= nil then
            for key, value in pairs(candidate) do
                if value ~= nil then
                    merged[key] = value
                end
            end
        end
    end
    return merged
end

function module.when(condition: boolean, props: any): any
    if condition then
        return props
    end
    return nil
end

function module.pick(name: string, variants: { [string]: any }): any
    local selected = variants[name]
    if selected == nil then
        error(("unknown variant `%s`"):format(name), 0)
    end
    return selected
end

return module
