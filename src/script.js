'use strict';

const { parse } = Reflect;

const builder = {
    literal(value, loc) {
        if (typeof value === 'object' && value !== null) {
            // regexp only
            var flags = '';
            if (value.global) flags += 'g';
            if (value.ignoreCase) flags += 'i';
            if (value.multiline) flags += 'm';
            if (value.sticky) flags += 'y';
            return {
                loc,
                type: 'Literal',
                value: null,
                regex: {
                    pattern: value.source,
                    flags
                }
            };
        } else {
            return {
                loc,
                type: 'Literal',
                value
            };
        }
    }
};

(source => {
    try {
        return Reflect.parse(source, {
            builder
        });
    } catch ({ message, fileName, lineNumber }) {
        return {
            type: 'Error',
            message,
            source: fileName,
            line: lineNumber
        };
    }
})
