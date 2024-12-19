export const errorConfigs = {
    400: {
        title: "Bad Request",
        message: "SYNTAX ERROR - INVALID REQUEST DETECTED",
        buttonText: "Reformat Request"
    },
    403: {
        title: "Forbidden",
        message: "ACCESS DENIED - SECURITY PROTOCOLS ENGAGED",
        buttonText: "Exit Restricted Area"
    },
    404: {
        title: "Not Found",
        message: "SYSTEM MALFUNCTION - PAGE NOT FOUND",
        buttonText: "Return to Matrix"
    },
    405: {
        title: "Method Not Allowed",
        message: "INVALID OPERATION - METHOD RESTRICTED",
        buttonText: "Change Protocol"
    },
    413: {
        title: "Payload Too Large",
        message: "OVERFLOW DETECTED - DATA EXCEEDS LIMITS",
        buttonText: "Reduce Payload"
    },
    500: {
        title: "Internal Server Error",
        message: "CRITICAL ERROR - SYSTEM CORE MALFUNCTION",
        buttonText: "System Reboot"
    }
};