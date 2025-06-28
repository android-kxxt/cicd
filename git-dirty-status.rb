#!/usr/bin/env ruby
require 'set'

untracked = Set.new()
deleted = Set.new()
modified = Set.new()
other = Set.new()

ARGF.each do |line|
    status, file = ( line.strip || line ).split(' ', 2)
    case status
    in '??'
        untracked << file
    in 'M'
        modified << file
    in 'D'
        deleted << file
    else
        other << file
    end
end

def markdown_code(input)
    escaped = input.gsub('`') { '\`' }
    return "`#{escaped}`"
end

clean = untracked.empty? && deleted.empty? && modified.empty? && other.empty?
if ! clean
    status = []
    { "Modified" => modified, "Untracked" => untracked, "Deleted" => deleted, "Other" => other }.each do |key, value|
        if ! value.empty? then
            status.append("#{key}: " + value.map(&method(:markdown_code)).join(', '))
        end
    end
    status = status.join ', '
    print "(Dirty, #{status})"
end